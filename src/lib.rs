//! LiveSplit auto splitter for Dandara (Unity 2018.4, legacy Mono).
//!
//! Classes and fields are looked up by name in `Assembly-CSharp`, so the same
//! code works for the Windows builds (`Dandara.exe`, 32-bit `mono.dll`; Steam,
//! Epic, also under Proton) and the 64-bit Linux build (`Dandara.x86_64`).
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
    Address, Process,
};

mod mono;
mod results_screen;
mod settings;

use crate::{
    mono::{field_or, Detected, MonoClass, Platform, Singleton},
    results_screen::ResultsScreens,
    settings::Settings,
};

asr::async_main!(stable);

/// Process names of the builds. `Dandara.x86` (32-bit Linux) is attached to
/// only to tell the user it isn't supported.
const PROCESS_NAMES: [&str; 3] = ["Dandara.exe", "Dandara.x86_64", "Dandara.x86"];

/// Scene the game starts in on "New Game".
const NEW_GAME_SCENE: &str = "A1_Void4";
/// `StoryEvent.Started`: a new game began.
const EVENT_STARTED: i32 = 0x1;
/// `StoryEvent.Ended`: the game was beaten (the final cutscene ended).
const EVENT_ENDED: i32 = 0x2;
/// A new game is recognised while `playedTime` is below this many seconds.
const NEW_GAME_WINDOW: f32 = 5.0;
/// Ticks per second; set explicitly since the waits below are in ticks.
const TICK_RATE: u32 = 120;
/// How long to wait for `StatisticsManager` to record the completion time
/// after `Ended` appears before falling back to `playedTime`.
const COMPLETION_TIME_WAIT_TICKS: u32 = TICK_RATE / 2;
/// How long to wait, once the statistics screen appears, for the game to
/// record the time it shows.
const SCREEN_TIME_WAIT_TICKS: u32 = TICK_RATE / 2;
/// How long after the game is beaten to give up on ever reading the
/// statistics screens and split right away.
const SCREENS_UNREADABLE_WAIT_TICKS: u32 = TICK_RATE / 2;
/// Split anyway if the statistics screen hasn't appeared this long after
/// the game is beaten: 10 minutes.
const RESULTS_SCREEN_TIMEOUT_TICKS: u32 = 10 * 60 * TICK_RATE;

async fn main() {
    asr::set_tick_rate(f64::from(TICK_RATE));
    let mut settings = Settings::register();

    loop {
        let (process, name) = attach().await;
        print_message(&format!("Attached to {name}"));

        process
            .until_closes(async {
                let platform = loop {
                    match Platform::detect(&process) {
                        Some(Detected::Supported(platform)) => break platform,
                        Some(Detected::Unsupported(what)) => {
                            print_message(&format!(
                                "{what} is not supported; on Linux, run Dandara.x86_64"
                            ));
                            // Nothing to do until the process closes.
                            core::future::pending::<()>().await;
                        }
                        None => wait_ticks(60).await,
                    }
                };
                print_message(&format!("Build: {}", platform.label));
                let (module, image) = attach_mono(&process).await;
                let mut game = Game::attach(&process, platform, &module, &image).await;
                let mut state = State::default();
                let mut run = Run::default();
                let mut last_timer_state = timer::state();
                let mut new_game = NewGameDetector::default();

                loop {
                    settings.update();
                    game.resolve_singletons(&process);
                    state.update(&process, &mut game);

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
                        if let Some(time) = run.split_time.take() {
                            // End on exactly the time the game shows on its statistics screen.
                            if !settings.remove_loading {
                                timer::set_game_time(Duration::seconds_f32(time));
                            }
                        }
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

async fn attach() -> (Process, &'static str) {
    loop {
        let attached = PROCESS_NAMES
            .into_iter()
            .find_map(|name| Process::attach(name).map(|process| (process, name)));
        if let Some(attached) = attached {
            return attached;
        }
        wait_ticks(60).await;
    }
}

async fn wait_ticks(n: u32) {
    for _ in 0..n {
        next_tick().await;
    }
}

/// Field offsets, looked up by name (falling back to the ASL's values).
/// All but `occurred_events` and `completion_time` are relative to the
/// `GameManager` instance.
struct Offsets {
    is_transitioning: u64,
    played_time: u64,
    current_scene: u64,
    base_camp_scene: u64,
    occurred_events: u64,
    /// `StatisticsManager._state.lastRunState.time`: the completion time the
    /// game shows after the credits.
    completion_time: Option<u64>,
}

/// The game classes the offsets are looked up in.
struct Classes {
    game_manager: MonoClass,
    game_state: MonoClass,
    story_manager: MonoClass,
    statistics_manager: MonoClass,
}

/// A singleton class and, once it has been validated, its `_instance` slot.
struct Manager {
    class: MonoClass,
    singleton: Option<Singleton>,
}

impl Manager {
    fn new(class: MonoClass) -> Self {
        Self {
            class,
            singleton: None,
        }
    }

    fn resolve(&mut self, process: &Process) {
        if self.singleton.is_none() {
            self.singleton = Singleton::resolve(process, &self.class);
        }
    }

    /// The current instance, or `None` while unresolved or null.
    fn instance(&self, process: &Process) -> Option<Address> {
        self.singleton.as_ref()?.instance(process)
    }
}

/// Where the game's data lives.
struct Game {
    platform: &'static Platform,
    game_manager: Manager,
    story_manager: Manager,
    statistics_manager: Manager,
    offsets: Offsets,
    results_screens: Option<ResultsScreens>,
    retry_in: u32,
}

impl Game {
    async fn attach(
        process: &Process,
        platform: &'static Platform,
        module: &Module,
        image: &Image,
    ) -> Self {
        let classes = Classes::find(process, platform, module, image).await;

        let mut last_missing = None;
        let offsets = loop {
            match Offsets::find(process, platform, module, image, &classes) {
                Ok(offsets) => break offsets,
                Err(field) => {
                    if last_missing != Some(field) {
                        last_missing = Some(field);
                        print_message(&format!("Waiting for field {field}"));
                    }
                    wait_ticks(TICK_RATE).await;
                }
            }
        };
        print_message("Classes found");

        let results_screens = MonoClass::find(process, platform, module, image, "StatisticsShow")
            .and_then(|show| {
                ResultsScreens::find(process, platform, &classes.statistics_manager, show)
            });
        if results_screens.is_none() {
            print_message("Statistics screens not found; the final split happens at Ended");
        }

        Self {
            platform,
            game_manager: Manager::new(classes.game_manager),
            story_manager: Manager::new(classes.story_manager),
            statistics_manager: Manager::new(classes.statistics_manager),
            offsets,
            results_screens,
            retry_in: 0,
        }
    }

    fn managers(&mut self) -> [&mut Manager; 3] {
        [
            &mut self.game_manager,
            &mut self.story_manager,
            &mut self.statistics_manager,
        ]
    }

    /// The singletons can only be validated once they exist, so retry about
    /// once a second until all are found.
    fn resolve_singletons(&mut self, process: &Process) {
        if self.managers().iter().all(|m| m.singleton.is_some()) {
            return;
        }
        if self.retry_in > 0 {
            self.retry_in -= 1;
            return;
        }
        self.retry_in = TICK_RATE;
        for manager in self.managers() {
            manager.resolve(process);
        }
    }
}

impl Classes {
    async fn find(
        process: &Process,
        platform: &'static Platform,
        module: &Module,
        image: &Image,
    ) -> Self {
        const NAMES: [&str; 4] = [
            "GameManager",
            "GameState",
            "StoryManager",
            "StatisticsManager",
        ];
        let mut reported = false;
        loop {
            let found = NAMES.map(|name| MonoClass::find(process, platform, module, image, name));
            if let [Some(game_manager), Some(game_state), Some(story_manager), Some(statistics_manager)] =
                found
            {
                return Self {
                    game_manager,
                    game_state,
                    story_manager,
                    statistics_manager,
                };
            }
            if !reported {
                reported = true;
                let status: Vec<_> = NAMES
                    .iter()
                    .zip(&found)
                    .map(|(name, class)| format!("{name}={}", class.is_some()))
                    .collect();
                print_message(&format!("Waiting for classes: {}", status.join(" ")));
            }
            wait_ticks(60).await;
        }
    }
}

impl Offsets {
    /// `Err` names the first field that couldn't be found.
    fn find(
        process: &Process,
        platform: &'static Platform,
        module: &Module,
        image: &Image,
        classes: &Classes,
    ) -> Result<Self, &'static str> {
        let header = platform.object_header();
        // `known` is the original ASL's offset, only valid for the 32-bit
        // Windows build.
        let field = |class: &MonoClass, name: &'static str, known: u64| {
            field_or(
                process,
                class,
                name,
                (!platform.is_64_bit()).then_some(known),
            )
            .ok_or(name)
        };

        // `GameState` is a struct embedded in `GameManager`. Mono stores struct
        // field offsets as if boxed (including the object header), so they are
        // rebased onto the embedded copy. The ASL's paths agree on 32-bit:
        // 0x38 + (0x1C - 8) = 0x4C for playedTime, and so on.
        let embedded = field(&classes.game_manager, "_gameState", 0x38)?;
        let in_game_state =
            |name, known| Ok(embedded + field(&classes.game_state, name, known)? - header);

        let offsets = Self {
            is_transitioning: field(&classes.game_manager, "_isTransitioning", 0x65)?,
            played_time: in_game_state("playedTime", 0x1C)?,
            current_scene: in_game_state("currentScene", 0xC)?,
            base_camp_scene: in_game_state("baseCampScene", 0x18)?,
            occurred_events: field(&classes.story_manager, "ocurredEvents", 0x10)?,
            completion_time: completion_time_offset(
                process,
                platform,
                module,
                image,
                &classes.statistics_manager,
            ),
        };
        if offsets.completion_time.is_none() {
            print_message("Completion time not found; the final split will use playedTime");
        }
        Ok(offsets)
    }
}

/// `StatisticsManager._state.lastRunState.time`, relative to the instance.
///
/// `GameManager.BeatTheGame` unlocks `StoryEvent.Ended` and then raises
/// `OnGameBeaten`, where `StatisticsManager` copies the current run (a struct,
/// so the copy is frozen) into `lastRunState`. `StatisticsShow` displays that
/// copy's `time` after the credits. Both `StatisticsState` and `RunState` are
/// structs embedded in their parent, so their offsets are rebased.
fn completion_time_offset(
    process: &Process,
    platform: &'static Platform,
    module: &Module,
    image: &Image,
    statistics_manager: &MonoClass,
) -> Option<u64> {
    let header = platform.object_header();
    let find = |class, field| {
        MonoClass::find(process, platform, module, image, class)?.field_offset(process, field)
    };
    let state = statistics_manager.field_offset(process, "_state")?;
    let last_run = find("StatisticsState", "lastRunState")?;
    let time = find("RunState", "time")?;
    Some(state + (last_run - header) + (time - header))
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
    /// The last completion time recorded by `StatisticsManager`.
    completion_time: Option<f32>,
    /// Started statistics screens showing a finished run; `None` if unreadable.
    results_screens: Option<Vec<Address>>,
    /// Whether each value could be read last tick, for logging transitions.
    readable: [Option<bool>; 5],
}

impl State {
    fn update(&mut self, process: &Process, game: &mut Game) {
        let off = &game.offsets;
        let p = game.platform;
        let game_manager = game.game_manager.instance(process);

        // On a failed read (menus, loading) keep the last known value.
        let played_time =
            game_manager.and_then(|gm| process.read::<f32>(gm + off.played_time).ok());
        let scene = game_manager.and_then(|gm| p.read_string(process, gm + off.current_scene));
        let base_camp =
            game_manager.and_then(|gm| p.read_string(process, gm + off.base_camp_scene));
        let is_transitioning =
            game_manager.and_then(|gm| process.read::<u8>(gm + off.is_transitioning).ok());
        let events = game
            .story_manager
            .instance(process)
            .and_then(|sm| p.read_ptr(process, sm + off.occurred_events))
            .and_then(|set| read_hash_set_i32(process, p, set));

        let statistics_manager = game.statistics_manager.instance(process);
        self.completion_time = statistics_manager
            .zip(off.completion_time)
            .and_then(|(st, offset)| process.read::<f32>(st + offset).ok());
        self.results_screens = statistics_manager
            .zip(game.results_screens.as_mut())
            .and_then(|(st, screens)| screens.read(process, st));

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
    finale: Finale,
    /// The time to end the run on, for the split being returned.
    split_time: Option<f32>,
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
        run.finale.last_completion = state.completion_time;
        run
    }

    fn should_split(&mut self, settings: &Settings, state: &State) -> bool {
        let mut split = false;
        let mut ended_now = false;

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
            if event == EVENT_ENDED {
                ended_now = true; // handled by the final split below
            } else {
                split |= settings.get_event_enabled(event);
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

        if settings.get_event_enabled(EVENT_ENDED) {
            if let Some(time) = self.finale.update(state, ended_now) {
                self.split_time = Some(time);
                split = true;
            }
        }
        split
    }
}

/// The final split: when the statistics screen appears after the credits,
/// on the time it shows.
///
/// The scene calls `BeatTheGame` both when the credits start and when the
/// statistics screen appears; each call copies the running play time into
/// `lastRunState`, and the screen shows the second copy.
#[derive(Default)]
struct Finale {
    phase: Phase,
    /// Last `lastRunState.time` seen; the game rewrites it when it's beaten.
    last_completion: Option<f32>,
    /// Results screens that existed before the game was beaten (e.g. from an
    /// earlier completion in this session); they don't trigger the final split.
    known_screens: Vec<Address>,
    /// Whether the results screens could ever be read.
    screens_readable: bool,
}

#[derive(Default)]
enum Phase {
    /// The game hasn't been beaten.
    #[default]
    Playing,
    /// `Ended` appeared for the first time this run; waiting (`ticks`) for
    /// its completion time to be recorded.
    Ended { ticks: u32 },
    /// The game was beaten (the credits started) at `time`, `ticks` ago.
    Beaten {
        time: f32,
        ticks: u32,
        /// Ticks since the statistics screen appeared, while waiting for
        /// the game to record the time it shows.
        screen_ticks: Option<u32>,
    },
    /// The final split happened.
    Done,
}

impl Phase {
    fn beaten(time: f32) -> Self {
        Self::Beaten {
            time,
            ticks: 0,
            screen_ticks: None,
        }
    }
}

impl Finale {
    /// The time to end the run on, once it's time for the final split.
    fn update(&mut self, state: &State, ended_now: bool) -> Option<f32> {
        let played_time = state.played_time();
        // A completion time is fresh if it matches the play time right now:
        // `BeatTheGame` copies the running time when the game is beaten. An
        // older one (from beating this save before) doesn't match.
        let fresh = |t: &f32| {
            played_time.is_some_and(|p| p > NEW_GAME_WINDOW && *t > 0.0 && (p - t).abs() < 2.0)
        };
        let completion = state.completion_time;
        let completion_changed = completion != self.last_completion;
        self.last_completion = completion;
        let screens = state.results_screens.as_deref();
        self.screens_readable |= screens.is_some();

        if let Phase::Playing | Phase::Ended { .. } = self.phase {
            if ended_now {
                self.phase = Phase::Ended { ticks: 0 };
            }
            if let Some(time) = completion.filter(|t| completion_changed && fresh(t)) {
                print_message(&format!("Game beaten, completion time: {time:.3}s"));
                self.phase = Phase::beaten(time);
            } else if let Phase::Ended { ticks } = &mut self.phase {
                *ticks += 1;
                if *ticks >= COMPLETION_TIME_WAIT_TICKS {
                    self.phase = match played_time {
                        // The completion time can't be read: use the play time.
                        Some(p) if completion.is_none() && p > NEW_GAME_WINDOW => {
                            print_message(&format!("Game beaten, play time: {p:.3}s"));
                            Phase::beaten(p)
                        }
                        _ => {
                            print_message(
                                "Ended was already in the loaded save; waiting for the game \
                                 to be beaten",
                            );
                            Phase::Playing
                        }
                    };
                }
            }
        }

        let split = match &mut self.phase {
            Phase::Playing | Phase::Ended { .. } => {
                for &s in screens.into_iter().flatten() {
                    if !self.known_screens.contains(&s) {
                        self.known_screens.push(s);
                    }
                }
                None
            }
            Phase::Beaten {
                time,
                ticks,
                screen_ticks,
            } => {
                *ticks += 1;
                let shown =
                    screens.is_some_and(|s| s.iter().any(|s| !self.known_screens.contains(s)));
                if shown {
                    screen_ticks.get_or_insert(0);
                }
                if let Some(screen_ticks) = screen_ticks {
                    *screen_ticks += 1;
                    // The time the screen shows: a fresh copy of the play time,
                    // made as it appears (allow a moment for it to be written).
                    let shown_time = completion.filter(fresh);
                    (shown_time.is_some() || *screen_ticks >= SCREEN_TIME_WAIT_TICKS).then(|| {
                        let time = shown_time.unwrap_or(*time);
                        print_message(&format!(
                            "Statistics screen shown: final split at {time:.3}s"
                        ));
                        time
                    })
                } else {
                    let reason =
                        if !self.screens_readable && *ticks >= SCREENS_UNREADABLE_WAIT_TICKS {
                            Some("Statistics screens can't be read: final split now")
                        } else if *ticks >= RESULTS_SCREEN_TIMEOUT_TICKS {
                            Some("Statistics screen not seen in time: final split")
                        } else {
                            None
                        };
                    reason.map(|reason| {
                        print_message(reason);
                        *time
                    })
                }
            }
            Phase::Done => None,
        };
        if split.is_some() {
            self.phase = Phase::Done;
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

// HashSet<T> (System.Core). Mono puts reference fields first, then the ints:
//     header, table, links, slots, comparer, si, touched, empty_slot, count, ...
// i.e. on 32-bit: links 0xC, slots 0x10, touched 0x1C (count 0x24, as in the ASL).
// Link: { int HashCode; int Next } — a slot is occupied when HashCode has HASH_FLAG.

/// `HashSet<T>` fields, in pointer-sized slots after the object header.
/// `touched` is the first int, right after the five references.
const HASH_SET_LINKS: u64 = 1;
const HASH_SET_SLOTS: u64 = 2;
const HASH_SET_TOUCHED: u64 = 5;
const HASH_FLAG: i32 = i32::MIN;
const MAX_EVENTS: usize = 1024;

/// Reads the occupied slots of a Mono `HashSet<int-sized T>`.
fn read_hash_set_i32(process: &Process, p: &Platform, set: Address) -> Option<Vec<i32>> {
    let field = |i| p.object_header() + i * p.pointer_bytes();
    let touched = process.read::<i32>(set + field(HASH_SET_TOUCHED)).ok()?;
    let touched = usize::try_from(touched).ok().filter(|&t| t <= MAX_EVENTS)?;
    if touched == 0 {
        return Some(Vec::new());
    }

    let links = p.read_ptr(process, set + field(HASH_SET_LINKS))?;
    let slots = p.read_ptr(process, set + field(HASH_SET_SLOTS))?;

    let mut link_buf = [[0i32; 2]; MAX_EVENTS];
    let mut slot_buf = [0i32; MAX_EVENTS];
    let link_buf = &mut link_buf[..touched];
    let slot_buf = &mut slot_buf[..touched];
    process
        .read_into_slice(links + p.array_data(), link_buf)
        .ok()?;
    process
        .read_into_slice(slots + p.array_data(), slot_buf)
        .ok()?;

    Some(
        link_buf
            .iter()
            .zip(slot_buf.iter())
            .filter(|([hash, _], _)| hash & HASH_FLAG != 0)
            .map(|(_, &event)| event)
            .collect(),
    )
}
