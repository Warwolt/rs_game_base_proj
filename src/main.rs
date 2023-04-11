extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;

mod game_state;
mod rendering;

use std::path::Path;
use std::str;

use configparser::ini::Ini;
use sdl2::keyboard::Keycode;
use sdl2::video::GLContext;
use sdl2::VideoSubsystem;
use sdl2::{
    event::Event,
    video::{GLProfile, Window},
};
use simple_logger::SimpleLogger;
use std::time::SystemTime;

const CONFIG_FILE: &str = "config.ini";

fn init_logging() {
    SimpleLogger::new().init().unwrap();
}

fn init_video(sdl: &sdl2::Sdl) -> sdl2::VideoSubsystem {
    let sdl_video = sdl.video().unwrap();

    // hint that we'll use the "330 core" OpenGL profile
    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_profile(GLProfile::Core);

    sdl_video
}

struct Monitor(i32);

fn init_window(
    sdl_video: &VideoSubsystem,
    title: &str,
    width: u32,
    height: u32,
    monitor: Monitor,
) -> Window {
    let mut window = sdl_video
        .window(title, width, height)
        .allow_highdpi()
        .opengl()
        .position_centered()
        .resizable()
        .hidden()
        .build()
        .unwrap();

    if let Ok(bounds) = sdl_video.display_bounds(monitor.0) {
        window.set_position(
            sdl2::video::WindowPos::Positioned(bounds.x + (bounds.w - width as i32) / 2),
            sdl2::video::WindowPos::Positioned(bounds.y + (bounds.h - height as i32) / 2),
        );
    }
    window.show();

    return window;
}

fn init_opengl(window: &Window) -> GLContext {
    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();
    window.subsystem().gl_set_swap_interval(1).unwrap();
    return gl_context;
}

fn main() {
    let window_width = 800;
    let window_height = 600;

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
    let sdl_video = init_video(&sdl);
    let window = init_window(
        &sdl_video,
        "Base Project",
        window_width,
        window_height,
        Monitor(1),
    );

    /* Initialize OpenGL */
    let _gl_context = init_opengl(&window); // closes on drop
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as _);

    /* Initialize ImGui */
    let mut imgui = imgui::Context::create();
    let mut imgui_sdl = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    let get_proc_address = |s| sdl_video.gl_get_proc_address(s) as _;
    let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, get_proc_address);

    /* Setup rendering */
    let mut game_renderer = rendering::Renderer::new();
    game_renderer.set_window_size(window_width, window_height);
    rendering::setup_shader_program(&mut game_renderer);

    let mut event_pump = sdl.event_pump().unwrap();
    let mut prev_time = SystemTime::now();
    let game_state = game_state::GameState {};
    'main_loop: loop {
        /* Input */
        let time_now = SystemTime::now();
        let _delta_time_ms = time_now.duration_since(prev_time).unwrap().as_millis();
        prev_time = time_now;

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

        /* Update */
        // let ui = game_renderer.frame();
        // // draw win95 button
        // {
        //     ui.set_draw_color(grey.r, grey.g, grey.b, grey.a);
        //     ui.draw_rect_fill(r.x, r.y, r.w, r.h);

        //     ui.set_draw_color(white.r, white.g, white.b, white.a);
        //     ui.draw_line(l1.x0, l1.y0, l1.x1, l1.y1);
        //     ui.draw_line(l2.x0, l2.y0, l2.x1, l2.y1);

        //     ui.set_draw_color(dark_grey.r, dark_grey.g, dark_grey.b, dark_grey.a);
        //     ui.draw_line(l3.x0, l3.y0, l3.x1, l3.y1);
        //     ui.draw_line(l4.x0, l4.y0, l4.x1, l4.y1);

        //     ui.set_draw_color(black.r, black.g, black.b, black.a);
        //     ui.draw_line(l5.x0, l5.y0, l5.x1, l5.y1);
        //     ui.draw_line(l6.x0, l6.y0, l6.x1, l6.y1);
        // }

        imgui_sdl.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let dev_ui = imgui.frame();
        if let Some(window) = dev_ui.window("Example Window").begin() {
            dev_ui.text("Hello Win95 Button");
            window.end();
        };

        /* Render */
        // game_renderer.render(&ui);
        game_renderer.render(&game_state);
        imgui_sdl.prepare_render(&dev_ui, &window);
        imgui_renderer.render(&mut imgui);

        window.gl_swap_window();
    }

    config.write(CONFIG_FILE).unwrap();
}
