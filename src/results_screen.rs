//! Detects the statistics screen that shows the completion time after the
//! credits.
//!
//! Every `StatisticsShow` component (the optional always-on timer, the
//! statistics screen after the credits, ...) registers itself in `Start` as a
//! listener of `StatisticsManager.onStatisticsChanged` and is never removed:
//!
//! ```text
//! StatisticsManager.onStatisticsChanged   UnityEvent
//!     .m_Calls                            InvokableCallList   (on UnityEventBase)
//!         .m_RuntimeCalls                 List<BaseInvokableCall> { _items, _size }
//!             [i] InvokableCall .Delegate UnityAction
//!                 .m_target               StatisticsShow      (on System.Delegate)
//!                     .type               RunType { Current, First, Last, Permadeath }
//! ```
//!
//! The screen after the credits shows a finished run (`type != Current`), so a
//! new listener of that kind appearing after `Ended` means it is on screen.

use asr::{print_message, Address, Process};

use crate::mono::{MonoClass, Platform};

/// `RunType` names, indexed by value. `Current` (0) is the live timer, not a
/// results screen.
const RUN_TYPE_NAMES: [&str; 4] = ["current", "first run", "last run", "permadeath"];
const RUN_TYPE_CURRENT: i32 = 0;
const MAX_LISTENERS: u64 = 64;
/// Cap on the per-class `Delegate` offset cache.
const MAX_LISTENER_CLASSES: usize = 16;

/// A `StatisticsShow` that has been started, and which run it displays.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Screen {
    object: Address,
    run_type: i32,
}

impl Screen {
    /// Shows a finished run (e.g. the screen after the credits).
    fn is_results(&self) -> bool {
        self.run_type != RUN_TYPE_CURRENT
    }

    fn run_type_name(&self) -> &'static str {
        usize::try_from(self.run_type)
            .ok()
            .and_then(|i| RUN_TYPE_NAMES.get(i))
            .copied()
            .unwrap_or("?")
    }
}

pub struct ResultsScreens {
    platform: &'static Platform,
    statistics_show: MonoClass,
    show_type: u64,
    event_field: u64,
    // Found from the live objects the first time they're seen.
    calls: Option<u64>,
    runtime_calls: Option<u64>,
    list: Option<(u64, u64)>,
    /// `InvokableCall.Delegate` per listener class (`None`: no such field).
    delegates: Vec<(Address, Option<u64>)>,
    target: Option<u64>,
    last_logged: Option<Vec<Screen>>,
}

impl ResultsScreens {
    pub fn find(
        process: &Process,
        platform: &'static Platform,
        statistics_manager: &MonoClass,
        statistics_show: MonoClass,
    ) -> Option<Self> {
        Some(Self {
            platform,
            show_type: statistics_show.field_offset(process, "type")?,
            event_field: statistics_manager.field_offset(process, "onStatisticsChanged")?,
            statistics_show,
            calls: None,
            runtime_calls: None,
            list: None,
            delegates: Vec::new(),
            target: None,
            last_logged: None,
        })
    }

    /// The `StatisticsShow` objects started so far that show a finished run.
    /// `None` if they couldn't be read.
    pub fn read(&mut self, process: &Process, statistics_manager: Address) -> Option<Vec<Address>> {
        let screens = self.read_all(process, statistics_manager)?;
        if self.last_logged.as_ref() != Some(&screens) {
            let names: Vec<_> = screens.iter().map(Screen::run_type_name).collect();
            print_message(&format!("Statistics screens: [{}]", names.join(", ")));
            self.last_logged = Some(screens.clone());
        }
        Some(
            screens
                .into_iter()
                .filter(Screen::is_results)
                .map(|s| s.object)
                .collect(),
        )
    }

    /// Every `StatisticsShow` started so far.
    fn read_all(&mut self, process: &Process, statistics_manager: Address) -> Option<Vec<Screen>> {
        let p = self.platform;
        let class = self.statistics_show;
        let event = p.read_ptr(process, statistics_manager + self.event_field)?;

        let calls_field = cached(&mut self.calls, || {
            class
                .of_object(process, event)?
                .field_offset_in_hierarchy(process, "m_Calls")
        })?;
        let calls = p.read_ptr(process, event + calls_field)?;

        let runtime_field = cached(&mut self.runtime_calls, || {
            class
                .of_object(process, calls)?
                .field_offset(process, "m_RuntimeCalls")
        })?;
        let list = p.read_ptr(process, calls + runtime_field)?;

        // List<T>: `_items` then `_size` (references first). The inflated
        // generic class may have no usable field info, hence the fallback.
        let (items_field, size_field) = cached(&mut self.list, || {
            let list_class = class.of_object(process, list)?;
            let header = p.object_header();
            Some((
                list_class.field_offset(process, "_items").unwrap_or(header),
                list_class
                    .field_offset(process, "_size")
                    .unwrap_or(header + p.pointer_bytes()),
            ))
        })?;
        let size = process.read::<i32>(list + size_field).ok()?;
        let size = u64::try_from(size).ok().filter(|&s| s <= MAX_LISTENERS)?;
        if size == 0 {
            return Some(Vec::new());
        }
        let items = p.read_ptr(process, list + items_field)?;
        if p.array_len(process, items)? < size {
            return None; // not a List layout we understand
        }

        let mut screens = Vec::new();
        for i in 0..size {
            let item_at = items + p.array_data() + i * p.pointer_bytes();
            let Some(item) = p.read_ptr(process, item_at) else {
                continue;
            };
            let Some(item_class) = class.of_object(process, item) else {
                continue;
            };
            let Some(delegate_field) = self.delegate_field(process, item_class) else {
                continue; // another kind of listener
            };
            let Some(delegate) = p.read_ptr(process, item + delegate_field) else {
                continue;
            };
            let Some(target_field) = cached(&mut self.target, || {
                class
                    .of_object(process, delegate)?
                    .field_offset_in_hierarchy(process, "m_target")
            }) else {
                continue;
            };
            let Some(target) = p.read_ptr(process, delegate + target_field) else {
                continue;
            };
            if !class.is_instance(process, target) {
                continue;
            }
            if let Ok(run_type) = process.read::<i32>(target + self.show_type) {
                screens.push(Screen {
                    object: target,
                    run_type,
                });
            }
        }
        Some(screens)
    }

    /// `Delegate` offset on a listener's class, cached per class so that a
    /// different kind of listener doesn't hide the `StatisticsShow` ones.
    fn delegate_field(&mut self, process: &Process, item_class: MonoClass) -> Option<u64> {
        let address = item_class.address();
        if let Some(&(_, offset)) = self.delegates.iter().find(|(c, _)| *c == address) {
            return offset;
        }
        let offset = item_class.field_offset(process, "Delegate");
        if self.delegates.len() < MAX_LISTENER_CLASSES {
            self.delegates.push((address, offset));
        }
        offset
    }
}

fn cached<T: Copy>(slot: &mut Option<T>, find: impl FnOnce() -> Option<T>) -> Option<T> {
    if slot.is_none() {
        *slot = find();
    }
    *slot
}
