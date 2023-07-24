use engine::Engine;
use engine::{
    geometry::Rect, graphics::rendering::Renderer, imgui::ImGui, input::config::ProgramConfig,
};
use sdl2::keyboard::Keycode;
use std::path::PathBuf;

use engine::imgui;

struct Game {}

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
    let game = Game {};

    /* Main loop */
    while !engine.should_quit() {
        /* Input */
        let sdl_events = engine.begin_frame();
        engine.handle_input(&sdl_events);
        imgui.handle_input(&sdl_events);

        /* Update */
        game.update(&mut engine, &mut imgui);
        engine.update();

        /* Render */
        game.render(&mut engine.renderer);
        engine.render();
        imgui.render();

        engine.end_frame();
    }

    config.show_dev_ui = imgui.is_visible();
    config.write_to_disk();
}

impl Game {
    pub fn update(&self, engine: &mut Engine, imgui: &mut ImGui) {
        if engine.input.keyboard.is_pressed_now(Keycode::F3) {
            imgui.toggle_visible();
        }

        // draw imgui ui
        let show_imgui = imgui.is_visible();
        let debug_ui = imgui.begin_frame(engine);
        if show_imgui {
            if let Some(debug_window) = debug_ui.window("Debug Window").begin() {
                if debug_ui.button("Press me!") {
                    log::debug!("button pressed");
                }
                debug_window.end();
            }
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
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
