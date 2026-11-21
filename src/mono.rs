//! Small, bounded reader for the Mono class metadata of Dandara's runtime
//! (Unity 2018.4, legacy `mono.dll`, 32-bit).
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
    game_engine::unity::mono::{Class, Image, Module},
    print_message,
    string::ArrayCString,
    Address, Address32, Process,
};

/// `MonoClass` offsets for the two layouts asr knows for 32-bit `mono.dll`.
struct Layout {
    label: &'static str,
    name: u64,
    field_count: u64,
    fields: u64,
    runtime_info: u64,
}

const LAYOUTS: [Layout; 2] = [
    Layout {
        label: "V1",
        name: 0x30,
        field_count: 0x64,
        fields: 0x74,
        runtime_info: 0xA4,
    },
    Layout {
        label: "V1Cattrs",
        name: 0x34,
        field_count: 0x68,
        fields: 0x78,
        runtime_info: 0xA8,
    },
];

const CLASS_PARENT: u64 = 0x24;
/// `MonoClassField`: { type, name, parent, offset }.
const FIELD_STRIDE: u64 = 0x10;
const FIELD_NAME: u64 = 0x4;
const FIELD_OFFSET: u64 = 0xC;
const MAX_FIELDS: u32 = 256;

pub fn read_ptr(process: &Process, address: Address) -> Option<Address> {
    process
        .read::<Address32>(address)
        .ok()
        .map(Address::from)
        .filter(|a| !a.is_null())
}

fn name_matches(process: &Process, name_ptr: Address, expected: &str) -> bool {
    read_ptr(process, name_ptr)
        .and_then(|s| process.read::<ArrayCString<64>>(s).ok())
        .is_some_and(|s| s.matches(expected))
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
}

impl MonoClass {
    /// Finds a class by name and works out which `MonoClass` layout applies by
    /// checking that the name pointer really points at `name`.
    pub fn find(process: &Process, module: &Module, image: &Image, name: &str) -> Option<Self> {
        let address = class_address(image.get_class(process, module, name)?);
        let layout = LAYOUTS
            .iter()
            .find(|l| name_matches(process, address + l.name, name))?;
        Some(Self { address, layout })
    }

    fn parent(&self, process: &Process) -> Option<Self> {
        Some(Self {
            address: read_ptr(process, self.address + CLASS_PARENT)?,
            layout: self.layout,
        })
    }

    /// Offset of a field declared on this class itself (no parent walk).
    /// Instance field offsets include the 8-byte object header.
    pub fn field_offset(&self, process: &Process, name: &str) -> Option<u32> {
        let count = process
            .read::<u32>(self.address + self.layout.field_count)
            .ok()
            .filter(|&c| c > 0 && c <= MAX_FIELDS)?;
        let fields = read_ptr(process, self.address + self.layout.fields)?;
        (0..count as u64)
            .map(|i| fields + i * FIELD_STRIDE)
            .find(|&f| name_matches(process, f + FIELD_NAME, name))
            .and_then(|f| process.read::<u32>(f + FIELD_OFFSET).ok())
    }

    /// `instance -> vtable -> klass == self`.
    fn is_instance(&self, process: &Process, instance: Address) -> bool {
        read_ptr(process, instance)
            .and_then(|vtable| read_ptr(process, vtable))
            .is_some_and(|klass| klass == self.address)
    }
}

/// `static T _instance` of the generic singleton base class of `class`.
pub struct Singleton {
    /// Address of the `_instance` slot in the parent's static data.
    slot: Address,
}

impl Singleton {
    /// Needs a live instance to validate against, so call it again until it
    /// succeeds (e.g. once per second).
    pub fn resolve(process: &Process, class: &MonoClass) -> Option<Self> {
        let parent = class.parent(process)?;
        // `_instance` is the only static field of both singleton bases, so 0
        // is also tried in case the inflated class has no usable field info.
        let looked_up = parent.field_offset(process, "_instance").map(u64::from);
        let fields = [looked_up, Some(0).filter(|_| looked_up != Some(0))];
        let runtime_info = read_ptr(process, parent.address + parent.layout.runtime_info)?;

        // MonoClassRuntimeInfo { u16 max_domain; MonoVTable *domain_vtables[] }:
        // Unity's scripts may run in a child domain, so try the first few.
        for domain_slot in [0x4u64, 0x8, 0xC] {
            let Some(vtable) = read_ptr(process, runtime_info + domain_slot) else {
                continue;
            };
            if read_ptr(process, vtable) != Some(parent.address) {
                continue; // not this class's vtable
            }
            // MonoVTable.data (static field storage); position varies by build.
            for data_offset in (0x4u64..0x40).step_by(4) {
                let Some(data) = read_ptr(process, vtable + data_offset) else {
                    continue;
                };
                for field in fields.into_iter().flatten() {
                    let slot = data + field;
                    if read_ptr(process, slot).is_some_and(|i| class.is_instance(process, i)) {
                        print_message(&format!(
                            "Singleton found (layout {}, domain slot {domain_slot:#x}, \
                             vtable data {data_offset:#x}, field {field:#x})",
                            parent.layout.label
                        ));
                        return Some(Self { slot });
                    }
                }
            }
        }
        None
    }

    /// The current instance, or `None` while it's null (e.g. title screen).
    pub fn instance(&self, process: &Process) -> Option<Address> {
        read_ptr(process, self.slot)
    }
}

/// Field offset by name, falling back to a known-good value (from the
/// original ASL pointer paths) with a log line if the lookup disagrees.
pub fn field_or(process: &Process, class: &MonoClass, name: &str, known: u32) -> u64 {
    match class.field_offset(process, name) {
        Some(offset) => {
            if offset != known {
                print_message(&format!(
                    "Note: {name} at {offset:#x} (ASL used {known:#x}), using {offset:#x}"
                ));
            }
            offset as u64
        }
        None => {
            print_message(&format!("Field {name} not found, using {known:#x}"));
            known as u64
        }
    }
}
