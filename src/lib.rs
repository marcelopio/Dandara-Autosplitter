use asr::{
    future::{next_tick, retry},
    print_message,
    settings::Gui,
    string::ArrayWString,
    time::Duration,
    timer::{self, TimerState},
    watcher::Watcher,
    Address, Address32, PointerSize, Process,
};

mod settings;

use crate::settings::Settings;

asr::async_main!(stable);

async fn main() {
    let mut settings = Settings::register();

    loop {
        // Hook to the process
        let process = retry(|| {
            ["Dandara.exe", "Dandara.x86_64"]
                .iter()
                .find_map(|&name| Process::attach(name))
        })
        .await;

        process
            .until_closes(async {
                // Detect version and get offsets
                let version = detect_version(&process).await;
                print_message(&format!("Detected version: {:?}", version));

                let mut watchers = Watchers::default();
                let mut vars = Vars::default();

                settings.update();

                loop {
                    // Update watchers
                    if !update_watchers(&process, &version, &mut watchers) {
                        next_tick().await;
                        continue;
                    }

                    let state = timer::state();

                    // Start logic
                    if state == TimerState::NotRunning {
                        if let (Some(played_time), Some(scene)) =
                            (&watchers.played_time.pair, &watchers.current_scene.pair)
                        {
                            if played_time.current > 0.0 && scene.current == "A1_Void4" {
                                print_message("Start");
                                timer::start();
                                vars.reset();
                            }
                        }
                    }

                    // Reset logic
                    if state == TimerState::Running || state == TimerState::Paused {
                        if let (Some(played_time), Some(scene)) =
                            (&watchers.played_time.pair, &watchers.current_scene.pair)
                        {
                            if played_time.current < 1.0
                                && played_time.old == 0.0
                                && scene.current == "A1_Void4"
                            {
                                print_message("Reset");
                                timer::reset();
                                vars.reset();
                            }
                        }
                    }

                    // Update game time
                    if let Some(played_time) = &watchers.played_time.pair {
                        timer::set_game_time(Duration::seconds_f32(played_time.current));
                    }

                    // Update loading state
                    if let Some(is_transitioning) = &watchers.is_transitioning.pair {
                        timer::pause_game_time();
                        if is_transitioning.current == 0 {
                            timer::resume_game_time();
                        }
                    }

                    // Split logic
                    if state == TimerState::Running {
                        let mut should_split = false;

                        // Check story events
                        if let Some(num_events) = &watchers.num_events.pair {
                            if num_events.changed() {
                                let events = read_story_events(&process, num_events.current);

                                let old_idx = num_events.old.max(0) as usize;
                                let current_idx = num_events.current.max(0) as usize;

                                if old_idx < current_idx && current_idx <= events.len() {
                                    for event in &events[old_idx..current_idx] {
                                        if let Some(event_name) = Settings::get_event_name(*event) {
                                            print_message(&format!(
                                                "Event occurred: {} ({:#x})",
                                                event_name, *event
                                            ));
                                        } else {
                                            print_message(&format!(
                                                "Unknown event occurred: {:#x}",
                                                *event
                                            ));
                                        }

                                        should_split =
                                            should_split || settings.get_event_enabled(*event);

                                        if *event == 0x2 {
                                            if let Some(played_time) = &watchers.played_time.pair {
                                                vars.start_credits_time = played_time.current;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Check scene changes
                        if let Some(scene) = &watchers.current_scene.pair {
                            if scene.changed() {
                                if !vars.current_scenes.contains(&scene.current) {
                                    print_message(&format!("Scene: {}", scene.current));
                                    vars.current_scenes.push(scene.current.clone());
                                    should_split = should_split
                                        || settings.get_scene_enabled(scene.current.as_str());
                                }
                            }
                        }

                        // Check for end credits split (25 seconds after credits start)
                        if vars.start_credits_time > 0.0 {
                            if let Some(played_time) = &watchers.played_time.pair {
                                if (played_time.current - vars.start_credits_time) > 25.0 {
                                    should_split = true;
                                    vars.start_credits_time = 0.0; // Reset to avoid re-splitting
                                }
                            }
                        }

                        if should_split {
                            timer::split();
                        }
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Version {
    Steam,
    Epic,
}

async fn detect_version(process: &Process) -> Version {
    if process.get_module_address("steamclient.dll").is_ok() {
        Version::Steam
    } else {
        Version::Epic
    }
}

#[derive(Default)]
struct Watchers {
    played_time: Watcher<f32>,
    current_scene: Watcher<String>,
    base_camp_scene: Watcher<String>,
    is_transitioning: Watcher<u8>,
    num_events: Watcher<i32>,
}

#[derive(Default)]
struct Vars {
    current_scenes: Vec<String>,
    start_credits_time: f32,
}

impl Vars {
    fn reset(&mut self) {
        self.current_scenes.clear();
        self.start_credits_time = 0.0;
    }
}

fn update_watchers(process: &Process, version: &Version, watchers: &mut Watchers) -> bool {
    let unity_base = match process.get_module_address("UnityPlayer.dll") {
        Ok(base) => base,
        Err(_) => return false,
    };

    match version {
        Version::Steam => {
            // float playedTime : "UnityPlayer.dll", 0x010A1120, 0x10, 0x7E0, 0x54, 0x7B4, 0x0, 0x4c
            process
                .read_pointer_path::<f32>(
                    unity_base,
                    PointerSize::Bit32,
                    &[0x010A1120, 0x10, 0x7E0, 0x54, 0x7B4, 0x0, 0x4c],
                )
                .ok()
                .map(|value| watchers.played_time.update_infallible(value));

            // string255 currentScene
            watchers.current_scene.update_infallible(read_unity_string(
                process,
                unity_base,
                &[0x010A1120, 0x10, 0x7E0, 0x54, 0x7B4, 0x0, 0x3c],
            ));

            // byte isTransitioning
            process
                .read_pointer_path::<u8>(
                    unity_base,
                    PointerSize::Bit32,
                    &[0x010A1120, 0x10, 0x7E0, 0x54, 0x7B4, 0x0, 0x65],
                )
                .ok()
                .map(|value| watchers.is_transitioning.update_infallible(value));

            // int numEvents
            process
                .read_pointer_path::<i32>(
                    unity_base,
                    PointerSize::Bit32,
                    &[0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10, 0x24],
                )
                .ok()
                .map(|value| watchers.num_events.update_infallible(value));
        }
        Version::Epic => {
            let mono_base = match process.get_module_address("mono.dll") {
                Ok(base) => base,
                Err(_) => return false,
            };

            // float playedTime : "mono.dll", 0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x4c
            watchers.played_time.update_infallible(
                match process.read_pointer_path::<f32>(
                    mono_base,
                    PointerSize::Bit32,
                    &[0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x4c],
                ) {
                    Ok(played_time) => played_time,
                    Err(_) => return false,
                },
            );

            // string255 currentScene
            watchers.current_scene.update_infallible(read_unity_string(
                process,
                mono_base,
                &[0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x3c, 0xc],
            ));

            // string255 baseCampScene
            watchers
                .base_camp_scene
                .update_infallible(read_unity_string(
                    process,
                    mono_base,
                    &[0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x48, 0xc],
                ));

            // byte isTransitioning
            process
                .read_pointer_path::<u8>(
                    mono_base,
                    PointerSize::Bit32,
                    &[0x001F72C4, 0x54, 0x280, 0x0, 0x8, 0xc8, 0x3c, 0x65],
                )
                .ok()
                .map(|value| watchers.is_transitioning.update_infallible(value));

            // int numEvents (uses UnityPlayer.dll for both versions)
            process
                .read_pointer_path::<i32>(
                    unity_base,
                    PointerSize::Bit32,
                    &[0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10, 0x24],
                )
                .ok()
                .map(|value| watchers.num_events.update_infallible(value));
        }
    }

    true
}

// Helper function to dereference a chain of offsets
fn deref_offsets<T: bytemuck::CheckedBitPattern>(
    process: &Process,
    base: Address,
    offsets: &[u64],
) -> Result<T, ()> {
    if offsets.is_empty() {
        return Err(());
    }

    let mut addr = base + offsets[0];

    // Follow each pointer in the chain
    for &offset in &offsets[1..] {
        // Read the pointer at the current address
        addr = process.read::<Address32>(addr).map_err(|_| ())?.into();
        // Add the next offset
        addr = addr + offset;
    }

    // Read the final value
    process.read::<T>(addr).map_err(|_| ())
}

fn read_unity_string(process: &Process, base: Address, offsets: &[u64]) -> String {
    return match process.read_pointer_path::<ArrayWString<128>>(base, PointerSize::Bit32, offsets) {
        Ok(array_str) => String::from_utf16(array_str.as_slice()).ok().unwrap(),
        Err(_) => "".to_string(),
    };
}

fn read_story_events(process: &Process, num_events: i32) -> Vec<i32> {
    let mut events = Vec::new();

    let unity_base = match process.get_module_address("UnityPlayer.dll") {
        Ok(base) => base,
        Err(_) => return events,
    };

    // Get array address: "UnityPlayer.dll", 0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10, 0x10
    let array_addr = match deref_offsets::<u32>(
        process,
        unity_base,
        &[0x0107DD14, 0xa8, 0x4, 0x8c, 0x54, 0x290, 0x10, 0x10, 0x10],
    ) {
        Ok(addr) => Address::from(addr),
        Err(_) => return events,
    };

    for i in 0..num_events {
        let offset = (i * 0x4 + 0x10) as u64;
        if let Ok(event) = process.read::<i32>(array_addr + offset) {
            events.push(event);
        }
    }

    events
}
