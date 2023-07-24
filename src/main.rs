use std::path::PathBuf;

use engine::imgui;
use engine::input::config::ProgramConfig;

fn init_logging() {
    simple_logger::SimpleLogger::new()
        .with_module_level("hot_lib_reloader", log::LevelFilter::Info)
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

fn main() {
    /* Initialize */
    init_logging();
    let mut config = init_config();
    let mut engine = engine::init(&config, 800, 600);
    let mut imgui = imgui::init(&mut engine, &config);

    /* Main loop */
    while !engine::should_quit(&engine) {
        /* Input */
        let sdl_events = engine::begin_frame(&mut engine);
        engine::handle_input(&mut engine, &sdl_events);
        imgui::handle_input(&mut imgui, &sdl_events);

        /* Update */
        game::update(&mut engine, &mut imgui, &mut config);
        engine::update(&mut engine);

        /* Render */
        game::render(&mut engine.renderer);
        engine::render(&mut engine.renderer);
        imgui::render(&mut imgui);

        engine::end_frame(&mut engine);
    }

    config.write_to_disk();
}

mod game {
    use engine::Engine;
    use engine::{
        geometry::Rect,
        graphics::rendering::Renderer,
        imgui::{self, ImGui},
        input::config::ProgramConfig,
    };
    use sdl2::keyboard::Keycode;

    pub fn update(engine: &mut Engine, imgui: &mut ImGui, config: &mut ProgramConfig) {
        if engine.input.keyboard.is_pressed_now(Keycode::F3) {
            imgui::toggle_visible(imgui);
        }

        config.show_dev_ui = imgui::is_visible(imgui);

        // draw imgui ui
        let show_imgui = imgui::is_visible(imgui);
        let debug_ui = imgui::begin_frame(imgui, engine);
        if show_imgui {
            if let Some(debug_window) = debug_ui.window("Debug Window").begin() {
                if debug_ui.button("Press me!") {
                    log::debug!("button pressed");
                }
                debug_window.end();
            }
        }
    }

    pub fn render(renderer: &mut Renderer) {
        // draw background
        renderer.clear();
        renderer.set_draw_color(0, 129, 129, 255);
        renderer.draw_rect_fill(Rect {
            x: 0,
            y: 0,
            w: renderer.canvas().dim.width,
            h: renderer.canvas().dim.height,
        });
    }
}
