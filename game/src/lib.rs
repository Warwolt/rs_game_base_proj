use engine::{geometry::Rect, graphics::rendering::Renderer, imgui::ImGui, Engine};
use sdl2::keyboard::Keycode;

pub struct GameState {}

/// Keep DLL global variables up to date with main globals
#[no_mangle]
pub fn set_global_contexts(
    logger: &'static dyn log::Log,
    level: log::LevelFilter,
    ctx: *mut imgui::sys::ImGuiContext,
) {
    log::set_max_level(level);
    log::set_logger(logger).unwrap();
    unsafe {
        imgui::sys::igSetCurrentContext(ctx);
    }
}

#[no_mangle]
pub fn init(
    logger: &'static dyn log::Log,
    level: log::LevelFilter,
    ctx: *mut imgui::sys::ImGuiContext,
) -> GameState {
    set_global_contexts(logger, level, ctx);
    GameState {}
}

#[no_mangle]
pub fn update(_state: &mut GameState, engine: &mut Engine, imgui: &mut ImGui) {
    if engine.input().keyboard.is_pressed_now(Keycode::F3) {
        imgui.toggle_visible();
    }

    if let Some(ui) = begin_imgui_frame(imgui, engine) {
        draw_debug_ui(ui);
    }
}

fn begin_imgui_frame<'a>(imgui: &'a mut ImGui, engine: &Engine<'a>) -> Option<&'a mut imgui::Ui> {
    let context_exists = unsafe { imgui::sys::igGetCurrentContext() != std::ptr::null_mut() };
    if context_exists {
        let imgui_is_visible = imgui.is_visible();
        let ui = imgui.begin_frame(engine);
        if imgui_is_visible {
            return Some(ui);
        }
    }
    None
}

fn draw_debug_ui(ui: &mut imgui::Ui) {
    if let Some(window) = ui.window("Debug Window").begin() {
        if ui.button("Press me!") {
            log::debug!("hello world!");
        }
        window.end();
    }
}

#[no_mangle]
pub fn render(_state: &mut GameState, renderer: &mut Renderer) {
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
