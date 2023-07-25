use sdl2::keyboard::Keycode;

use crate::game;

pub fn update(engine: &engine::Engine) {
    if cfg!(debug_assertions) {
        if engine.input.keyboard.is_pressed_now(Keycode::F5) {
            rebuild_game_lib();
        }

        if game::was_reloaded() {
            on_game_reloaded();
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
