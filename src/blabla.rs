use std::fmt;

use asr::{
    async_main,
    future::next_tick,
    game_engine::unity::{mono::Module, scene_manager::SceneManager},
    print_message, Error, PointerSize, Process,
};

async_main!(stable);

async fn main() {
    print_message("Hello, World!!!");
    // TODO: Set up some general state and settings.
    loop {
        let process = Process::wait_attach("Dandara.exe").await;
        process
            .until_closes(async {
                print_message("Starting");

                let module = Module::wait_attach_auto_detect(&process).await;
                print_message("Module");

                let image = module.wait_get_default_image(&process).await;
                print_message("Image");

                let game_manager = &image.wait_get_class(&process, &module, "GameManager").await;
                print_message("Got GameManager");

                let game_manager_static_instance = game_manager
                    .wait_get_static_instance(&process, &module, "_instance")
                    .await;
                print_message(&format!(
                    "game_manager_static_instance: {:p}",
                    game_manager_static_instance
                ));

                let game_manager_table =
                    game_manager.wait_get_static_table(&process, &module).await;
                print_message(&format!("static table: {:p}", game_manager_table));

                let field_offset = game_manager
                    .wait_get_field_offset(&process, &module, "_instance")
                    .await;
                let singleton_location = game_manager_table + field_offset;

                print_message(&format!("field offset: {:x}", field_offset));

                let is_paused_offset = game_manager
                    .wait_get_field_offset(&process, &module, "_isPaused")
                    .await;

                print_message(&format!("is paused offset, {}", is_paused_offset));

                loop {
                    let instance_game_manager =
                        process.read_pointer(singleton_location, PointerSize::Bit64);
                        
                    if instance_game_manager.is_ok() {
                        
                        let instance_game_manager = instance_game_manager.unwrap();
                        print_message("Got the instance");

                        let is_paused = process.read::<u8>(instance_game_manager.add(is_paused_offset as u64)).ok().unwrap();
                        print_message(&format!("Is paused: {}", is_paused));                        

                    }

                    next_tick().await;
                }
            })
            .await;
    }
}
