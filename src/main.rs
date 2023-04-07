use std::path::Path;

use configparser::ini::Ini;
use glow::HasContext;
use imgui::Context;
use imgui_glow_renderer::AutoRenderer;
use imgui_sdl2_support::SdlPlatform;
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

fn init_glow_context(window: &Window) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
    }
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
    let _gl_context = init_opengl(&window); // closes on drop
    let glow_context = init_glow_context(&window);

    /* Initialize ImGui */
    let mut imgui_context = Context::create();
    imgui_context.set_ini_filename(None);
    imgui_context.set_log_filename(None);
    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);
    let mut imgui_platform = SdlPlatform::init(&mut imgui_context);

    let mut event_pump = sdl.event_pump().unwrap();
    let mut renderer = AutoRenderer::initialize(glow_context, &mut imgui_context).unwrap();
    'main_loop: loop {
        /* Input */
        for event in event_pump.poll_iter() {
            imgui_platform.handle_event(&mut imgui_context, &event);

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
        imgui_platform.prepare_frame(&mut imgui_context, &window, &event_pump);
        let ui = imgui_context.new_frame();
        if let Some(window) = ui.window("Example Window").begin() {
            ui.text("Window is visible");
            window.end();
        };

        /* Game render */
        unsafe {
            let gl = renderer.gl_context();
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.clear_color(0.0, 0.5, 0.5, 0.0);
        }

        /* ImGui render */
        let imgui_draw_data = imgui_context.render();
        renderer.render(imgui_draw_data).unwrap();

        window.gl_swap_window();
    }

    config.write(CONFIG_FILE).unwrap();
}
