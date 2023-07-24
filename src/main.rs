use engine::input::config::ProgramConfig;
use std::path::PathBuf;

#[hot_lib_reloader::hot_module(dylib = "game")]
mod game {
    pub use game::GameState;
    // these usages should mirror the ones of game/src/lib.rs
    pub use engine::{geometry::Rect, graphics::rendering::Renderer, imgui::ImGui, Engine};
    pub use sdl2::keyboard::Keycode;

    hot_functions_from_file!("game/src/lib.rs");

    #[lib_updated]
    pub fn was_reloaded() -> bool {}
}

fn init_logging() {
    simple_logger::SimpleLogger::new()
        .with_module_level("hot_lib_reloader", log::LevelFilter::Error)
        .init()
        .unwrap();
}

fn init_config() -> ProgramConfig {
    let mut config = ProgramConfig::from_file(&PathBuf::from("config.ini"));

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--monitor" {
        config.monitor = args[2].parse::<u64>().unwrap();
    }

    config
}

fn init_game() -> game::GameState {
    unsafe {
        game::init(
            log::logger(),
            log::max_level(),
            imgui::sys::igGetCurrentContext(),
        )
    }
}

fn on_game_reloaded() {
    unsafe {
        log::info!("Game code hot-reloaded");
        game::set_global_contexts(
            log::logger(),
            log::max_level(),
            imgui::sys::igGetCurrentContext(),
        );
    }
}

fn main() {
    /* Initialize */
    init_logging();
    let mut config = init_config();
    let mut engine = engine::init(&config, 800, 600);
    let mut imgui = engine::imgui::init(&mut engine, &config);
    let mut game = init_game();

    /* Main loop */
    while !engine.should_quit() {
        if game::was_reloaded() {
            on_game_reloaded();
        }

        /* Input */
        let sdl_events = engine.begin_frame();
        engine.handle_input(&sdl_events);
        imgui.handle_input(&sdl_events);

        /* Update */
        game::update(&mut game, &mut engine, &mut imgui);
        engine.update();

        /* Render */
        game::render(&mut game, &mut engine.renderer());
        engine.render();
        imgui.render();

        engine.end_frame();
    }

    config.show_dev_ui = imgui.is_visible();
    config.write_to_disk();
}
