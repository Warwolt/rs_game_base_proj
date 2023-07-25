use sdl2::video::GLContext;

use crate::{input::config::ProgramConfig, Engine};

pub struct ImGui {
    imgui: imgui::Context,
    imgui_sdl: imgui_sdl2::ImguiSdl2,
    imgui_renderer: imgui_opengl_renderer::Renderer,
    is_visible: bool,
}

pub fn init_imgui(engine: &Engine, config: &ProgramConfig) -> ImGui {
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

impl ImGui {
    pub fn begin_frame(&mut self, engine: &Engine) -> &mut imgui::Ui {
        self.imgui_sdl.prepare_frame(
            self.imgui.io_mut(),
            &engine.window,
            &engine.sdl_event_pump.mouse_state(),
        );
        let frame = self.imgui.frame();
        self.imgui_sdl.prepare_render(&frame, &engine.window);
        frame
    }

    pub fn handle_input(&mut self, events: &Vec<sdl2::event::Event>) {
        for event in events {
            self.imgui_sdl.handle_event(&mut self.imgui, event);
        }
    }

    pub fn render(&mut self, _gl: &GLContext) {
        self.imgui_renderer.render(&mut self.imgui);
    }

    pub fn toggle_visible(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
}
