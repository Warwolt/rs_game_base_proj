use crate::{input::config::ProgramConfig, Engine};

pub struct ImGui {
    imgui: imgui::Context,
    imgui_sdl: imgui_sdl2::ImguiSdl2,
    imgui_renderer: imgui_opengl_renderer::Renderer,
    is_visible: bool,
}

pub fn init(engine: &Engine, config: &ProgramConfig) -> ImGui {
    let mut imgui = imgui::Context::create();
    let imgui_sdl = imgui_sdl2::ImguiSdl2::new(&mut imgui, &engine.window);
    let get_proc_address = |s| engine.sdl_video.gl_get_proc_address(s) as _;
    let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, get_proc_address);
    log::info!("ImGui initialied");

    ImGui {
        imgui,
        imgui_sdl,
        imgui_renderer,
        is_visible: config.show_dev_ui,
    }
}

pub fn begin_frame<'a>(imgui: &'a mut ImGui, engine: &'a Engine<'a>) -> &'a mut imgui::Ui {
    imgui.imgui_sdl.prepare_frame(
        imgui.imgui.io_mut(),
        &engine.window,
        &engine.sdl_event_pump.mouse_state(),
    );
    let frame = imgui.imgui.frame();
    imgui.imgui_sdl.prepare_render(&frame, &engine.window);
    frame
}

pub fn handle_input(imgui: &mut ImGui, events: &Vec<sdl2::event::Event>) {
    for event in events {
        imgui.imgui_sdl.handle_event(&mut imgui.imgui, event);
    }
}

pub fn render(imgui: &mut ImGui) {
    imgui.imgui_renderer.render(&mut imgui.imgui);
}

pub fn toggle_visible(imgui: &mut ImGui) {
    imgui.is_visible = !imgui.is_visible;
}

pub fn is_visible(imgui: &ImGui) -> bool {
    imgui.is_visible
}
