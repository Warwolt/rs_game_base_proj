use std::path::PathBuf;

use engine::{geometry::Rect, input::config::ProgramConfig};
use sdl2::keyboard::Keycode;

fn main() {
    /* Initialize logging */
    engine::init_logging();
    log::info!("Program start");

    /* Initialize configuration */
    let mut config = ProgramConfig::from_file(&PathBuf::from("config.ini"));

    /* Parse args */
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--monitor" {
        config.monitor = args[2].parse::<u64>().unwrap();
    }

    /* Initialize SDL and ImGui */
    let mut engine = engine::Engine::new(&config, 800, 600);
    let mut imgui = engine::ImGui::new(&mut engine);

    /* Main loop */
    'main_loop: loop {
        /* Input */
        let sdl_events = engine.begin_frame();
        engine.handle_input(&sdl_events);
        imgui.handle_input(&sdl_events);

        /* Update */
        engine.update();
        if engine.should_quit() {
            break 'main_loop;
        }

        // app update
        {
            if engine.input.keyboard.is_pressed_now(Keycode::F3) {
                config.show_dev_ui = !config.show_dev_ui;
            }

            // draw imgui ui
            let debug_ui = imgui.begin_frame(&engine);
            if config.show_dev_ui {
                if let Some(debug_window) = debug_ui.window("Debug Window").begin() {
                    if debug_ui.button("Press me!") {
                        log::debug!("button pressed");
                    }
                    debug_window.end();
                }
            }
        }

        /* Render */
        // app render
        {
            // draw background
            engine.renderer.clear();
            engine.renderer.set_draw_color(0, 129, 129, 255);
            engine.renderer.draw_rect_fill(Rect {
                x: 0,
                y: 0,
                w: engine.renderer.canvas().dim.width,
                h: engine.renderer.canvas().dim.height,
            });
        }

        engine.renderer.render();
        imgui.imgui_renderer.render(&mut imgui.imgui);

        engine.window.gl_swap_window();
    }

    config.write_to_disk();
}
