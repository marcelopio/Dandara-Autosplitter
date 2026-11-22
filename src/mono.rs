//! Small, bounded reader for the Mono class metadata and objects of Dandara's
//! runtime (Unity 2018.4, legacy Mono). Supports the Windows build
//! (`mono.dll`, 32-bit) and the 64-bit Linux build (`libmono.so`).
//!
//! asr's Mono helpers are still used to find the runtime, the
//! `Assembly-CSharp` image and classes by name. This module does the rest
//! itself, because asr couldn't resolve the static `_instance` of the game's
//! generic singleton base classes (`PersistentSingleton<GameManager>`,
//! `Singleton<StoryManager>`), and its field lookup walks into those parents.
//!
//! Every step is validated against back-pointers (vtable -> class,
//! instance -> vtable -> class), so a wrong guess is rejected instead of
//! producing garbage.

use asr::{
    file_format::{elf, pe},
    game_engine::unity::mono::{Class, Image, Module},
    print_message,
    string::ArrayCString,
    Address, PointerSize, Process,
};

/// `MonoClass` offsets (as in asr's tables for this Mono version).
struct Layout {
    label: &'static str,
    parent: u64,
    name: u64,
    field_count: u64,
    fields: u64,
    runtime_info: u64,
}

/// Everything that differs between the supported builds.
pub struct Platform {
    pub label: &'static str,
    pointer_size: PointerSize,
    /// `V1` and `V1Cattrs`; the right one is picked per class by name check.
    layouts: [Layout; 2],
    /// `MonoClassField` { type, name, parent, offset }: stride and offsets.
    field_stride: u64,
    field_name: u64,
    field_offset: u64,
}

const PE_32: Platform = Platform {
    label: "Windows (32-bit)",
    pointer_size: PointerSize::Bit32,
    layouts: [
        Layout {
            label: "V1",
            parent: 0x24,
            name: 0x30,
            field_count: 0x64,
            fields: 0x74,
            runtime_info: 0xA4,
        },
        Layout {
            label: "V1Cattrs",
            parent: 0x24,
            name: 0x34,
            field_count: 0x68,
            fields: 0x78,
            runtime_info: 0xA8,
        },
    ],
    field_stride: 0x10,
    field_name: 0x4,
    field_offset: 0xC,
};

const ELF_64: Platform = Platform {
    label: "Linux (64-bit)",
    pointer_size: PointerSize::Bit64,
    layouts: [
        Layout {
            label: "V1",
            parent: 0x28,
            name: 0x40,
            field_count: 0x8C,
            fields: 0xA0,
            runtime_info: 0xF0,
        },
        Layout {
            label: "V1Cattrs",
            parent: 0x28,
            name: 0x48,
            field_count: 0x94,
            fields: 0xA8,
            runtime_info: 0xF8,
        },
    ],
    field_stride: 0x20,
    field_name: 0x8,
    field_offset: 0x18,
};

const MAX_FIELDS: u32 = 256;
const MAX_STRING_LEN: usize = 128;
const MAX_HIERARCHY_DEPTH: usize = 6;

pub enum Detected {
    Supported(&'static Platform),
    /// Mono is loaded, but asr can't read this kind of build.
    Unsupported(&'static str),
}

impl Platform {
    /// Works out the build from the loaded Mono runtime. `None` until it's loaded.
    pub fn detect(process: &Process) -> Option<Detected> {
        if let Ok(mono) = process.get_module_address("mono.dll") {
            return Some(
                match pe::MachineType::read(process, mono).and_then(pe::MachineType::pointer_size) {
                    Some(PointerSize::Bit32) => Detected::Supported(&PE_32),
                    _ => Detected::Unsupported("64-bit mono.dll"),
                },
            );
        }
        if let Ok(mono) = process.get_module_address("libmono.so") {
            return Some(match elf::pointer_size(process, mono) {
                Some(PointerSize::Bit64) => Detected::Supported(&ELF_64),
                _ => Detected::Unsupported("32-bit Linux build (Dandara.x86)"),
            });
        }
        None
    }

    pub fn is_64_bit(&self) -> bool {
        self.pointer_size == PointerSize::Bit64
    }

    pub fn pointer_bytes(&self) -> u64 {
        // The discriminants are the widths in bytes.
        self.pointer_size as u64
    }

    /// `MonoObject` header (vtable + sync).
    pub fn object_header(&self) -> u64 {
        2 * self.pointer_bytes()
    }

    /// Start of the elements of a `MonoArray` (vtable, sync, bounds, max_length).
    pub fn array_data(&self) -> u64 {
        4 * self.pointer_bytes()
    }

    /// `MonoArray.max_length` (the capacity).
    pub fn array_len(&self, process: &Process, array: Address) -> Option<u64> {
        // A pointer-sized integer, so read it as a pointer.
        process
            .read_pointer(array + 3 * self.pointer_bytes(), self.pointer_size)
            .ok()
            .map(Address::value)
    }

    /// Reads a non-null pointer.
    pub fn read_ptr(&self, process: &Process, address: Address) -> Option<Address> {
        process
            .read_pointer(address, self.pointer_size)
            .ok()
            .filter(|a| !a.is_null())
    }

    /// Reads a `string` stored at `field`. `Some("")` for null, `None` if unreadable.
    pub fn read_string(&self, process: &Process, field: Address) -> Option<String> {
        let string = process.read_pointer(field, self.pointer_size).ok()?;
        if string.is_null() {
            return Some(String::new());
        }
        // MonoString: header, then `int length`, then the UTF-16 chars.
        let length_at = string + self.object_header();
        let len = process.read::<i32>(length_at).ok()?;
        let len = usize::try_from(len).ok().filter(|&l| l <= MAX_STRING_LEN)?;
        let mut buf = [0u16; MAX_STRING_LEN];
        process
            .read_into_slice(length_at + 4, &mut buf[..len])
            .ok()?;
        Some(String::from_utf16_lossy(&buf[..len]))
    }

    fn name_matches(&self, process: &Process, name_ptr: Address, expected: &str) -> bool {
        self.read_ptr(process, name_ptr)
            .and_then(|s| process.read::<ArrayCString<64>>(s).ok())
            .is_some_and(|s| s.matches(expected))
    }
}

/// The address behind asr's `Class` handle.
fn class_address(class: Class) -> Address {
    // SAFETY: `Class` is a `Copy` struct whose only field is an `Address`
    // (asr `game_engine/unity/mono/class.rs`); the sizes are checked at
    // compile time by `transmute`.
    unsafe { core::mem::transmute::<Class, Address>(class) }
}

#[derive(Clone, Copy)]
pub struct MonoClass {
    address: Address,
    layout: &'static Layout,
    platform: &'static Platform,
}

impl MonoClass {
    /// Finds a class by name and works out which `MonoClass` layout applies by
    /// checking that the name pointer really points at `name`.
    pub fn find(
        process: &Process,
        platform: &'static Platform,
        module: &Module,
        image: &Image,
        name: &str,
    ) -> Option<Self> {
        let address = class_address(image.get_class(process, module, name)?);
        let layout = platform
            .layouts
            .iter()
            .find(|l| platform.name_matches(process, address + l.name, name))?;
        Some(Self {
            address,
            layout,
            platform,
        })
    }

    /// The class of a live object (`object -> vtable -> klass`).
    pub fn of_object(&self, process: &Process, object: Address) -> Option<Self> {
        let p = self.platform;
        let vtable = p.read_ptr(process, object)?;
        Some(Self {
            address: p.read_ptr(process, vtable)?,
            ..*self
        })
    }

    fn parent(&self, process: &Process) -> Option<Self> {
        Some(Self {
            address: self
                .platform
                .read_ptr(process, self.address + self.layout.parent)?,
            ..*self
        })
    }

    /// Offset of a field declared on this class itself (no parent walk).
    /// Instance field offsets include the object header.
    pub fn field_offset(&self, process: &Process, name: &str) -> Option<u64> {
        let p = self.platform;
        let count = process
            .read::<u32>(self.address + self.layout.field_count)
            .ok()
            .filter(|&c| c > 0 && c <= MAX_FIELDS)?;
        let fields = p.read_ptr(process, self.address + self.layout.fields)?;
        (0..u64::from(count))
            .map(|i| fields + i * p.field_stride)
            .find(|&f| p.name_matches(process, f + p.field_name, name))
            .and_then(|f| process.read::<u32>(f + p.field_offset).ok())
            .map(u64::from)
    }

    /// Like [`Self::field_offset`], but also searches the parent classes (for
    /// fields declared on a base class, e.g. `Delegate.m_target`). Only use it
    /// on non-generic classes.
    pub fn field_offset_in_hierarchy(&self, process: &Process, name: &str) -> Option<u64> {
        let mut class = *self;
        for _ in 0..MAX_HIERARCHY_DEPTH {
            if let Some(offset) = class.field_offset(process, name) {
                return Some(offset);
            }
            class = class.parent(process)?;
        }
        None
    }

    pub fn address(&self) -> Address {
        self.address
    }

    /// `instance -> vtable -> klass == self`.
    pub fn is_instance(&self, process: &Process, instance: Address) -> bool {
        self.of_object(process, instance)
            .is_some_and(|class| class.address == self.address)
    }
}

/// `static T _instance` of the generic singleton base class of `class`.
pub struct Singleton {
    /// Address of the `_instance` slot in the parent's static data.
    slot: Address,
    platform: &'static Platform,
}

impl Singleton {
    /// Needs a live instance to validate against, so call it again until it
    /// succeeds (e.g. once per second).
    pub fn resolve(process: &Process, class: &MonoClass) -> Option<Self> {
        let p = class.platform;
        let ptr = p.pointer_bytes();
        let parent = class.parent(process)?;
        // `_instance` is the only static field of both singleton bases, so 0
        // is also tried in case the inflated class has no usable field info.
        let looked_up = parent.field_offset(process, "_instance");
        let fields = [looked_up, (looked_up != Some(0)).then_some(0)];
        let runtime_info = p.read_ptr(process, parent.address + parent.layout.runtime_info)?;

        // MonoClassRuntimeInfo { u16 max_domain; MonoVTable *domain_vtables[] }:
        // Unity's scripts may run in a child domain, so try the first few.
        for domain_slot in [ptr, 2 * ptr, 3 * ptr] {
            let Some(vtable) = p.read_ptr(process, runtime_info + domain_slot) else {
                continue;
            };
            if p.read_ptr(process, vtable) != Some(parent.address) {
                continue; // not this class's vtable
            }
            // MonoVTable.data (static field storage); position varies by build.
            for data_offset in (ptr..16 * ptr).step_by(ptr as usize) {
                let Some(data) = p.read_ptr(process, vtable + data_offset) else {
                    continue;
                };
                for field in fields.into_iter().flatten() {
                    let slot = data + field;
                    if p.read_ptr(process, slot)
                        .is_some_and(|i| class.is_instance(process, i))
                    {
                        print_message(&format!(
                            "Singleton found (layout {}, domain slot {domain_slot:#x}, \
                             vtable data {data_offset:#x}, field {field:#x})",
                            parent.layout.label
                        ));
                        return Some(Self { slot, platform: p });
                    }
                }
            }
        }
        None
    }

    /// The current instance, or `None` while it's null (e.g. title screen).
    pub fn instance(&self, process: &Process) -> Option<Address> {
        self.platform.read_ptr(process, self.slot)
    }
}

/// Field offset by name. `known` is a fallback that's only valid for one
/// build (the original ASL's 32-bit Windows offsets); a disagreement or the
/// use of the fallback is logged, a field that can't be found at all is not.
pub fn field_or(
    process: &Process,
    class: &MonoClass,
    name: &str,
    known: Option<u64>,
) -> Option<u64> {
    match (class.field_offset(process, name), known) {
        (Some(offset), Some(known)) if offset != known => {
            print_message(&format!(
                "Note: {name} at {offset:#x} (ASL used {known:#x}), using {offset:#x}"
            ));
            Some(offset)
        }
        (Some(offset), _) => Some(offset),
        (None, Some(known)) => {
            print_message(&format!("Field {name} not found, using {known:#x}"));
            Some(known)
        }
        // The caller reports it (it may retry, and shouldn't log every time).
        (None, None) => None,
    }
}
