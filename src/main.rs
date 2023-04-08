extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;

use std::path::Path;

use configparser::ini::Ini;
use sdl2::keyboard::Keycode;
use sdl2::video::GLContext;
use sdl2::VideoSubsystem;
use sdl2::{
    event::Event,
    video::{GLProfile, Window},
};
use simple_logger::SimpleLogger;

const CONFIG_FILE: &str = "config.ini";

fn init_logging() {
    SimpleLogger::new().init().unwrap();
}

fn init_video(sdl_video: &VideoSubsystem) {
    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);
}

fn init_window(sdl_video: &VideoSubsystem, title: &str, width: u32, height: u32) -> Window {
    let mut window = sdl_video
        .window(title, width, height)
        .allow_highdpi()
        .opengl()
        .position_centered()
        .resizable()
        .hidden()
        .build()
        .unwrap();

    // hack: move window to 2nd monitor because my laptop is stupid
    {
        let bounds = sdl_video.display_bounds(1).unwrap();
        window.set_position(
            sdl2::video::WindowPos::Positioned(bounds.x + (bounds.w - width as i32) / 2),
            sdl2::video::WindowPos::Positioned(bounds.y + (bounds.h - height as i32) / 2),
        );
        window.show();
    }

    return window;
}

fn init_opengl(window: &Window) -> GLContext {
    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();
    window.subsystem().gl_set_swap_interval(1).unwrap();
    return gl_context;
}

fn main() {
    /* Initialize logging */
    init_logging();

    /* Initialize configuration */
    let mut config = Ini::new();
    if Path::new("./config.ini").exists() {
        config.load("config.ini").unwrap();
    }
    config.set("numbers", "first", Some(String::from("11")));

    /* Initialize SDL */
    let sdl = sdl2::init().unwrap();
    let sdl_video = sdl.video().unwrap();
    init_video(&sdl_video);
    let window = init_window(&sdl_video, "Base Project", 800, 600);

    /* Initialize OpenGL */
    let _gl_context = init_opengl(&window); // closes on drop
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as _);

    /* Initialize ImGui */
    let mut imgui = imgui::Context::create();
    let mut imgui_sdl = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    let imgui_renderer =
        imgui_opengl_renderer::Renderer::new(&mut imgui, |s| sdl_video.gl_get_proc_address(s) as _);

    let mut event_pump = sdl.event_pump().unwrap();
    'main_loop: loop {
        /* Input */
        for event in event_pump.poll_iter() {
            imgui_sdl.handle_event(&mut imgui, &event);
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(Keycode::Escape) => {
                        break 'main_loop;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        /* Game update */
        // ..

        /* ImGui update */
        imgui_sdl.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let ui = imgui.frame();
        if let Some(window) = ui.window("Example Window").begin() {
            ui.text("Window is visible");
            window.end();
        };

        /* Render */
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        /* Game render */

        /* ImGui render */
        imgui_sdl.prepare_render(&ui, &window);
        imgui_renderer.render(&mut imgui);

        window.gl_swap_window();
    }

    config.write(CONFIG_FILE).unwrap();
}
