//! LiveSplit auto splitter for Dandara (Unity 2018.4, legacy Mono, 32-bit).
//!
//! Classes and fields are looked up by name in `Assembly-CSharp`, so the same
//! code works for every store build that ships `mono.dll` (Steam, Epic).
//!
//! Layout (from `Assembly-CSharp.dll`):
//!
//! ```text
//! GameManager  : PersistentSingleton<GameManager>   (static _instance lives on the parent)
//!     GameState _gameState          (a struct, stored inline)
//!         string currentScene, baseCampScene; float playedTime
//!     bool _isTransitioning
//! StoryManager : Singleton<StoryManager>
//!     HashSet<StoryEvent> ocurredEvents
//! ```

use std::collections::HashSet;

use asr::{
    future::next_tick,
    game_engine::unity::mono::{Image, Module, Version},
    print_message,
    settings::Gui,
    time::Duration,
    timer::{self, TimerState},
    watcher::Watcher,
    Address, Address32, Process,
};

mod mono;
mod settings;

use crate::{
    mono::{field_or, read_ptr, MonoClass, Singleton},
    settings::Settings,
};

asr::async_main!(stable);

/// Scene the game starts in on "New Game".
const NEW_GAME_SCENE: &str = "A1_Void4";
/// `StoryEvent.Started`: a new game began.
const EVENT_STARTED: i32 = 0x1;
/// `StoryEvent.Ended`: the credits started.
const EVENT_ENDED: i32 = 0x2;
/// A new game is recognised while `playedTime` is below this many seconds.
const NEW_GAME_WINDOW: f32 = 5.0;
/// Seconds after the credits start at which the optional final split happens.
const END_CREDITS_DELAY: f32 = 25.0;

async fn main() {
    let mut settings = Settings::register();

    loop {
        let process = Process::wait_attach("Dandara.exe").await;
        print_message("Attached to Dandara.exe");

        process
            .until_closes(async {
                let (module, image) = attach_mono(&process).await;
                let mut game = Game::attach(&process, &module, &image).await;
                let mut state = State::default();
                let mut run = Run::default();
                let mut last_timer_state = timer::state();
                let mut new_game = NewGameDetector::default();

                loop {
                    settings.update();
                    game.resolve_singletons(&process);
                    state.update(&process, &game);

                    let timer_state = timer::state();
                    let new_game = new_game.update(&state);

                    // Reset first so a new game can immediately start a new run.
                    if new_game
                        && settings.auto_reset
                        && matches!(timer_state, TimerState::Running | TimerState::Paused)
                    {
                        print_message("Reset");
                        timer::reset();
                    }

                    if new_game && settings.auto_start && timer::state() == TimerState::NotRunning {
                        print_message("Start");
                        timer::start();
                        // Baseline on what's in memory (it may still be the previous
                        // save for a tick), but let `Started` split if enabled.
                        run = Run::seeded_from(&state);
                        run.seen_events.remove(&EVENT_STARTED);
                        last_timer_state = TimerState::Running;
                    }

                    // The run was started some other way (manually, or auto start is off):
                    // treat what already happened as the baseline instead of splitting on it.
                    let timer_state = timer::state();
                    if last_timer_state == TimerState::NotRunning
                        && timer_state == TimerState::Running
                    {
                        run = Run::seeded_from(&state);
                    }
                    last_timer_state = timer_state;

                    if matches!(timer_state, TimerState::Running | TimerState::Paused) {
                        update_game_time(&settings, &state);
                    }

                    if timer_state == TimerState::Running && run.should_split(&settings, &state) {
                        timer::split();
                    }

                    if let Some(scene) = &state.scene.pair {
                        if scene.changed() {
                            print_message(&format!(
                                "Scene: {:?} -> {:?} (playedTime {:.2})",
                                scene.old,
                                scene.current,
                                state.played_time().unwrap_or(-1.0)
                            ));
                            timer::set_variable("Scene", &scene.current);
                        }
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}

/// Attaches to the Mono runtime and returns it with the `Assembly-CSharp` image.
///
/// A version only counts once `GameManager` can be found in the image, which
/// protects against succeeding before the game assembly is loaded.
async fn attach_mono(process: &Process) -> (Module, Image) {
    loop {
        let candidates = [
            Module::attach_auto_detect(process),
            Module::attach(process, Version::V1),
            Module::attach(process, Version::V1Cattrs),
        ];
        for (module, label) in candidates.into_iter().zip(["auto", "V1", "V1Cattrs"]) {
            let Some(module) = module else { continue };
            let Some(image) = module.get_default_image(process) else {
                continue;
            };
            if image.get_class(process, &module, "GameManager").is_some() {
                print_message(&format!("Mono attached ({label})"));
                return (module, image);
            }
        }
        // Retrying constantly is costly (it walks the whole class cache), so back off.
        wait_ticks(60).await;
    }
}

async fn wait_ticks(n: u32) {
    for _ in 0..n {
        next_tick().await;
    }
}

/// Field offsets, looked up by name (falling back to the ASL's values).
/// All but `occurred_events` are relative to the `GameManager` instance.
struct Offsets {
    is_transitioning: u64,
    played_time: u64,
    current_scene: u64,
    base_camp_scene: u64,
    occurred_events: u64,
}

/// Where the game's data lives.
struct Game {
    game_manager: MonoClass,
    story_manager: MonoClass,
    game_manager_instance: Option<Singleton>,
    story_manager_instance: Option<Singleton>,
    offsets: Offsets,
    retry_in: u32,
}

impl Game {
    async fn attach(process: &Process, module: &Module, image: &Image) -> Self {
        let mut reported = false;
        let (game_manager, game_state_class, story_manager) = loop {
            let find = |name| MonoClass::find(process, module, image, name);
            match (find("GameManager"), find("GameState"), find("StoryManager")) {
                (Some(gm), Some(gs), Some(sm)) => break (gm, gs, sm),
                found => {
                    if !reported {
                        reported = true;
                        print_message(&format!(
                            "Waiting for classes: GameManager={} GameState={} StoryManager={}",
                            found.0.is_some(),
                            found.1.is_some(),
                            found.2.is_some()
                        ));
                    }
                    wait_ticks(60).await;
                }
            }
        };

        // `GameState` is a struct embedded in `GameManager`. Mono stores struct
        // field offsets as if boxed (including the object header), so they are
        // rebased onto the embedded copy. The ASL's paths agree:
        // 0x38 + (0x1C - 8) = 0x4C for playedTime, and so on.
        let game_state = field_or(process, &game_manager, "_gameState", 0x38);
        let in_game_state = |name, known| {
            game_state + field_or(process, &game_state_class, name, known) - OBJECT_HEADER
        };
        let offsets = Offsets {
            is_transitioning: field_or(process, &game_manager, "_isTransitioning", 0x65),
            played_time: in_game_state("playedTime", 0x1C),
            current_scene: in_game_state("currentScene", 0xC),
            base_camp_scene: in_game_state("baseCampScene", 0x18),
            occurred_events: field_or(process, &story_manager, "ocurredEvents", 0x10),
        };
        print_message("Classes found");

        Self {
            game_manager,
            story_manager,
            game_manager_instance: None,
            story_manager_instance: None,
            offsets,
            retry_in: 0,
        }
    }

    /// The singletons can only be validated once they exist, so retry about
    /// once a second until both are found.
    fn resolve_singletons(&mut self, process: &Process) {
        if self.game_manager_instance.is_some() && self.story_manager_instance.is_some() {
            return;
        }
        if self.retry_in > 0 {
            self.retry_in -= 1;
            return;
        }
        self.retry_in = 120;
        if self.game_manager_instance.is_none() {
            self.game_manager_instance = Singleton::resolve(process, &self.game_manager);
        }
        if self.story_manager_instance.is_none() {
            self.story_manager_instance = Singleton::resolve(process, &self.story_manager);
        }
    }
}

/// Values read from the game this tick.
#[derive(Default)]
struct State {
    played_time: Watcher<f32>,
    scene: Watcher<String>,
    base_camp: Watcher<String>,
    is_transitioning: Watcher<bool>,
    /// Story events currently present in `StoryManager.ocurredEvents`.
    events: Vec<i32>,
    /// Whether each value could be read last tick, for logging transitions.
    readable: [Option<bool>; 5],
}

impl State {
    fn update(&mut self, process: &Process, game: &Game) {
        let off = &game.offsets;
        let game_manager = game
            .game_manager_instance
            .as_ref()
            .and_then(|s| s.instance(process));

        // On a failed read (menus, loading) keep the last known value.
        let played_time =
            game_manager.and_then(|gm| process.read::<f32>(gm + off.played_time).ok());
        let scene = game_manager.and_then(|gm| read_mono_string(process, gm + off.current_scene));
        let base_camp =
            game_manager.and_then(|gm| read_mono_string(process, gm + off.base_camp_scene));
        let is_transitioning =
            game_manager.and_then(|gm| process.read::<u8>(gm + off.is_transitioning).ok());
        let events = game
            .story_manager_instance
            .as_ref()
            .and_then(|s| s.instance(process))
            .and_then(|sm| read_ptr(process, sm + off.occurred_events))
            .and_then(|set| read_hash_set_i32(process, set));

        self.log_readable(0, "playedTime", played_time.is_some());
        self.log_readable(1, "currentScene", scene.is_some());
        self.log_readable(2, "baseCampScene", base_camp.is_some());
        self.log_readable(3, "isTransitioning", is_transitioning.is_some());
        self.log_readable(4, "ocurredEvents", events.is_some());

        if let Some(v) = played_time {
            self.played_time.update_infallible(v);
        }
        if let Some(v) = scene {
            self.scene.update_infallible(v);
        }
        if let Some(v) = base_camp {
            self.base_camp.update_infallible(v);
        }
        if let Some(v) = is_transitioning {
            self.is_transitioning.update_infallible(v != 0);
        }
        if let Some(v) = events {
            // The set keeps insertion order, so the last new entry is the latest event.
            if let Some(&latest) = v.iter().rev().find(|e| !self.events.contains(e)) {
                let name = Settings::get_event_name(latest).unwrap_or("Unknown");
                timer::set_variable("Last event", &format!("{name} ({latest:#x})"));
            }
            self.events = v;
        }
    }

    fn log_readable(&mut self, slot: usize, name: &str, ok: bool) {
        if self.readable[slot] != Some(ok) {
            self.readable[slot] = Some(ok);
            print_message(&format!(
                "{name}: {}",
                if ok { "readable" } else { "not readable" }
            ));
        }
    }

    fn played_time(&self) -> Option<f32> {
        self.played_time.pair.map(|p| p.current)
    }
}

/// Recognises the start of a new game: being in the first scene with a
/// `playedTime` that is small. Fires once, then re-arms when `playedTime`
/// goes backwards (another save/new game) or grows past the window.
///
/// It deliberately doesn't require catching `playedTime == 0` exactly: the
/// game state is often unreadable while a new game loads, so the first
/// successful read can already be a few frames in.
struct NewGameDetector {
    armed: bool,
}

impl Default for NewGameDetector {
    fn default() -> Self {
        Self { armed: true }
    }
}

impl NewGameDetector {
    fn update(&mut self, state: &State) -> bool {
        let (Some(time), Some(scene)) = (&state.played_time.pair, &state.scene.pair) else {
            return false;
        };
        if time.current < time.old || time.current >= NEW_GAME_WINDOW {
            self.armed = true;
        }
        let in_new_game =
            scene.current == NEW_GAME_SCENE && time.current > 0.0 && time.current < NEW_GAME_WINDOW;
        if self.armed && in_new_game {
            self.armed = false;
            print_message(&format!(
                "New game detected (playedTime {:.2})",
                time.current
            ));
            return true;
        }
        false
    }
}

/// Per-run bookkeeping. Cleared whenever a run starts.
#[derive(Default)]
struct Run {
    seen_events: HashSet<i32>,
    visited_scenes: HashSet<String>,
    visited_camps: HashSet<String>,
    credits_started_at: Option<f32>,
}

impl Run {
    /// A run that considers everything currently in the game as already done.
    fn seeded_from(state: &State) -> Self {
        let mut run = Self::default();
        run.seen_events.extend(state.events.iter().copied());
        if let Some(scene) = &state.scene.pair {
            run.visited_scenes.insert(scene.current.clone());
        }
        if let Some(camp) = &state.base_camp.pair {
            run.visited_camps.insert(camp.current.clone());
        }
        run
    }

    fn should_split(&mut self, settings: &Settings, state: &State) -> bool {
        let mut split = false;

        // Story events: split the first time each enabled event happens in this run.
        // Using a set (not the event count) means reloading a save never re-splits.
        for &event in &state.events {
            if !self.seen_events.insert(event) {
                continue;
            }
            match Settings::get_event_name(event) {
                Some(name) => print_message(&format!("Event: {name} ({event:#x})")),
                None => print_message(&format!("Unknown event: {event:#x}")),
            }
            split |= settings.get_event_enabled(event);

            if event == EVENT_ENDED && settings.split_end_credits {
                self.credits_started_at = state.played_time();
            }
        }

        // Scenes: split on the first visit to each enabled scene.
        if let Some(scene) = &state.scene.pair {
            if scene.changed()
                && !scene.current.is_empty()
                && self.visited_scenes.insert(scene.current.clone())
            {
                split |= settings.get_scene_enabled(&scene.current);
            }
        }

        // Camps: split the first time the player saves at each camp.
        if let Some(camp) = &state.base_camp.pair {
            if camp.changed()
                && !camp.current.is_empty()
                && self.visited_camps.insert(camp.current.clone())
            {
                print_message(&format!("New camp: {}", camp.current));
                split |= settings.split_new_camp;
            }
        }

        // Final split a fixed time after the credits start.
        if let (Some(started), Some(now)) = (self.credits_started_at, state.played_time()) {
            if now - started > END_CREDITS_DELAY {
                self.credits_started_at = None;
                split = true;
            }
        }

        split
    }
}

fn update_game_time(settings: &Settings, state: &State) {
    if settings.remove_loading {
        // Real time minus room transitions.
        match state.is_transitioning.pair.map(|p| p.current) {
            Some(true) => timer::pause_game_time(),
            _ => timer::resume_game_time(),
        }
    } else if let Some(time) = state.played_time() {
        // The game's own play timer drives game time; keep LiveSplit from
        // advancing it between ticks.
        timer::pause_game_time();
        timer::set_game_time(Duration::seconds_f32(time));
    }
}

// --- Raw Mono object layouts (32-bit) -------------------------------------
//
// MonoString: vtable 0x0, sync 0x4, length 0x8, chars 0xC
// MonoArray:  vtable 0x0, sync 0x4, bounds 0x8, max_length 0xC, data 0x10
// HashSet<T> (System.Core, Mono puts reference fields first):
//     table 0x8, links 0xC, slots 0x10, comparer 0x14, si 0x18,
//     touched 0x1C, empty_slot 0x20, count 0x24, threshold 0x28, generation 0x2C
// Link: { int HashCode; int Next } — a slot is occupied when HashCode has HASH_FLAG.

/// MonoObject header (vtable + sync) on 32-bit.
const OBJECT_HEADER: u64 = 0x8;
const STRING_LENGTH: u64 = 0x8;
const STRING_CHARS: u64 = 0xC;
const ARRAY_DATA: u64 = 0x10;
const HASH_SET_LINKS: u64 = 0xC;
const HASH_SET_SLOTS: u64 = 0x10;
const HASH_SET_TOUCHED: u64 = 0x1C;
const HASH_FLAG: i32 = i32::MIN;

const MAX_STRING_LEN: usize = 128;
const MAX_EVENTS: usize = 1024;

/// Reads a `string` stored at `field`. `Some("")` for null, `None` if unreadable.
fn read_mono_string(process: &Process, field: Address) -> Option<String> {
    let string: Address = process.read::<Address32>(field).ok()?.into();
    if string.is_null() {
        return Some(String::new());
    }
    let len = process.read::<i32>(string + STRING_LENGTH).ok()?;
    let len = usize::try_from(len).ok().filter(|&l| l <= MAX_STRING_LEN)?;
    let mut buf = [0u16; MAX_STRING_LEN];
    process
        .read_into_slice(string + STRING_CHARS, &mut buf[..len])
        .ok()?;
    Some(String::from_utf16_lossy(&buf[..len]))
}

/// Reads the occupied slots of a Mono `HashSet<int-sized T>`.
fn read_hash_set_i32(process: &Process, set: Address) -> Option<Vec<i32>> {
    if set.is_null() {
        return Some(Vec::new());
    }
    let touched = process.read::<i32>(set + HASH_SET_TOUCHED).ok()?;
    let touched = usize::try_from(touched).ok().filter(|&t| t <= MAX_EVENTS)?;
    if touched == 0 {
        return Some(Vec::new());
    }

    let links: Address = process.read::<Address32>(set + HASH_SET_LINKS).ok()?.into();
    let slots: Address = process.read::<Address32>(set + HASH_SET_SLOTS).ok()?.into();

    let mut link_buf = [[0i32; 2]; MAX_EVENTS];
    let mut slot_buf = [0i32; MAX_EVENTS];
    let link_buf = &mut link_buf[..touched];
    let slot_buf = &mut slot_buf[..touched];
    process.read_into_slice(links + ARRAY_DATA, link_buf).ok()?;
    process.read_into_slice(slots + ARRAY_DATA, slot_buf).ok()?;

    Some(
        link_buf
            .iter()
            .zip(slot_buf.iter())
            .filter(|([hash, _], _)| hash & HASH_FLAG != 0)
            .map(|(_, &event)| event)
            .collect(),
    )
}
