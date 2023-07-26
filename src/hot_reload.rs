use sdl2::keyboard::Keycode;

use crate::game;
use crate::WINDOW_TITLE;

pub fn update(engine: &mut engine::Engine) {
    if cfg!(debug_assertions) {
        if engine.input.keyboard.is_pressed_now(Keycode::F5) {
            let _ = engine
                .window
                .set_title(&format!("{} (reloading...)", WINDOW_TITLE));
            rebuild_game_lib();
        }

        if game::was_reloaded() {
            on_game_reloaded();
            let _ = engine.window.set_title(WINDOW_TITLE);
        }
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

fn rebuild_game_lib() {
    log::info!("Rebuilding game code");
    let _ = std::process::Command::new("cargo")
        .args(["build", "-p", "game"])
        .spawn()
        .expect("failed to execute process");
}
