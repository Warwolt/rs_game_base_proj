extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;

mod input;
mod midpoint;
mod rendering;

use std::path::Path;
use std::str;

use configparser::ini::Ini;
use input::InputDevices;
use rendering::Renderer;
use sdl2::keyboard::Keycode;
use sdl2::video::GLContext;
use sdl2::VideoSubsystem;
use sdl2::{
    event::Event,
    video::{GLProfile, Window},
};
use simple_logger::SimpleLogger;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

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

#[rustfmt::skip]
fn draw_button(renderer: &mut Renderer, rect: Rect, pressed: bool) {
    let white = (255, 255, 255);
    let light_grey = (223, 223, 223);
    let grey = (194, 194, 194);
    let dark_grey = (129, 129, 129);
    let black = (0, 0, 0);

    let top_outline = if pressed { black } else { white };
    let top_highlight = if pressed { dark_grey } else { light_grey };
    let bottom_outline = if pressed { black } else { black };
    let bottom_highlight = if pressed { light_grey } else { dark_grey };

    // button body
    renderer.set_draw_color(grey.0, grey.1, grey.2, 255);
    renderer.draw_rect_fill(rect.x, rect.y, rect.w, rect.h);

    // top outline
    renderer.set_draw_color(top_outline.0, top_outline.1, top_outline.2, 255);
    renderer.draw_line(rect.x, rect.y, rect.x, rect.y + rect.h);
    renderer.draw_line(rect.x, rect.y, rect.x + rect.w, rect.y);

    // top highlight
    renderer.set_draw_color(top_highlight.0, top_highlight.1, top_highlight.2, 255);
    renderer.draw_line(rect.x + 1, rect.y + 1, rect.x + 1, rect.y + rect.h - 1);
    renderer.draw_line(rect.x + 1, rect.y + 1, rect.x + rect.w - 1, rect.y);

    // bottom outline
    renderer.set_draw_color(bottom_outline.0, bottom_outline.1, bottom_outline.2, 255);
    renderer.draw_line(rect.x + rect.w, rect.y, rect.x + rect.w, rect.y + rect.h);
    renderer.draw_line(rect.x, rect.y + rect.h, rect.x + rect.w, rect.y + rect.h);

    // bottom highlight
    renderer.set_draw_color(bottom_highlight.0, bottom_highlight.1, bottom_highlight.2, 255);
    renderer.draw_line(rect.x + rect.w - 1, rect.y + 1, rect.x + rect.w - 1, rect.y + rect.h - 1);
    renderer.draw_line(rect.x + 1, rect.y + rect.h - 1, rect.x + rect.w - 1, rect.y + rect.h - 1);
}

fn draw_smiley(renderer: &mut Renderer, window_width: u32, window_height: u32) {
    let (win_center_x, win_center_y) = ((window_width / 2) as i32, (window_height / 2) as i32);

    let smiley_center_x = win_center_x + 250;
    let smiley_center_y = win_center_y - 150;

    // body
    renderer.set_draw_color(255, 216, 102, 255);
    renderer.draw_fill_circle(smiley_center_x, smiley_center_y, 100);

    // eyes
    renderer.set_draw_color(255, 255, 255, 255);
    renderer.draw_fill_circle(smiley_center_x - 50, smiley_center_y - 20, 20);
    renderer.draw_fill_circle(smiley_center_x + 50, smiley_center_y - 20, 20);

    // pupils
    renderer.set_draw_color(0, 0, 0, 255);
    renderer.draw_fill_circle(smiley_center_x - 50, smiley_center_y - 20, 5);
    renderer.draw_fill_circle(smiley_center_x + 50, smiley_center_y - 20, 5);

    // mouth
    renderer.draw_line(
        smiley_center_x - 30,
        smiley_center_y + 20,
        smiley_center_x,
        smiley_center_y + 40,
    );
    renderer.draw_line(
        smiley_center_x + 30,
        smiley_center_y + 20,
        smiley_center_x,
        smiley_center_y + 40,
    );

    // pretend hit box
    renderer.set_draw_color(0, 255, 0, 255);
    renderer.draw_rect(smiley_center_x - 100, smiley_center_y - 100, 200, 200);
}

fn point_is_inside_rect(point: glam::IVec2, rect: Rect) -> bool {
    let (rect_x0, rect_y0, rect_x1, rect_y1) = (rect.x, rect.y, rect.x + rect.w, rect.y + rect.h);
    let (point_x, point_y) = (point.x as i32, point.y as i32);
    let horizontal_overlap = rect_x0 <= point_x && point_x <= rect_x1;
    let vertical_overlap = rect_y0 <= point_y && point_y <= rect_y1;

    horizontal_overlap && vertical_overlap
}

fn main() {
    let window_width = 800;
    let window_height = 600;

    /* Initialize logging */
    init_logging();

    /* Initialize configuration */
    // TODO: setup a way of serializing/deserializing our own config struct
    // instead of working directly with this `Ini` value
    let mut config = Ini::new();
    if Path::new("./config.ini").exists() {
        // loading existing file
        config.load("config.ini").unwrap();
    } else {
        // set defaults on new file
        config.set("Imgui", "Show", Some(String::from("false")));
    }

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

    /* Setup input */
    let mut input = InputDevices::new();

    /* Setup rendering */
    let mut renderer = rendering::Renderer::new();
    renderer.on_window_resize(window_width, window_height);

    let mut event_pump = sdl.event_pump().unwrap();
    let mut prev_time = SystemTime::now();
    let mut show_dev_ui = config.getbool("Imgui", "Show").unwrap().unwrap();
    let mut alpha_amount = 255;
    'main_loop: loop {
        /* Input */
        let time_now = SystemTime::now();
        let _delta_time_ms = time_now.duration_since(prev_time).unwrap().as_millis();
        prev_time = time_now;

        for event in event_pump.poll_iter() {
            imgui_sdl.handle_event(&mut imgui, &event);
            input.mouse.handle_event(&event);

            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::KeyDown { keycode, .. } => {
                    if keycode == Some(Keycode::Escape) {
                        break 'main_loop;
                    }
                    if keycode == Some(Keycode::F3) {
                        show_dev_ui = !show_dev_ui;
                    }
                }
                _ => {}
            }
        }
        input.mouse.update();

        /* Update */
        renderer.clear();

        // draw background
        renderer.set_draw_color(0, 129, 129, 255);
        renderer.draw_rect_fill(0, 0, window_width as i32, window_height as i32);

        let button_width = 75;
        let button_height = 23;
        let button_rect = Rect {
            w: button_width,
            h: button_height,
            x: (window_width as i32 - button_width) / 2,
            y: (window_height as i32 - button_height) / 2,
        };

        let button_pressed = point_is_inside_rect(input.mouse.pos, button_rect)
            && input.mouse.left_button.is_pressed();

        draw_button(&mut renderer, button_rect, button_pressed);
        draw_smiley(&mut renderer, window_width, window_height);

        // transparency test
        renderer.set_draw_color(0, 255, 0, alpha_amount);
        renderer.draw_line(100 - 32, 100 - 32 - 5, 100 - 32 + 64 + 64, 100 - 32 - 5);
        renderer.draw_circle(100, 100, 32);
        renderer.draw_fill_circle(170, 100, 32);
        renderer.draw_rect(100 - 32, 170 - 32, 64, 64);
        renderer.draw_rect_fill(170 - 32, 170 - 32, 64, 64);

        imgui_sdl.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let dev_ui = imgui.frame();
        if show_dev_ui {
            if let Some(window) = dev_ui.window("Example Window").begin() {
                dev_ui.text("Hello Win95 Button");
                dev_ui.text(format!(
                    "Mouse pos: ({},{})",
                    input.mouse.pos.x, input.mouse.pos.y
                ));
                dev_ui.text(format!(
                    "Mouse left button pressed: {}",
                    input.mouse.left_button.is_pressed()
                ));
                dev_ui.slider("Alpha", 0, 255, &mut alpha_amount);
                window.end();
            };
        }

        /* Render */
        renderer.render();
        imgui_sdl.prepare_render(&dev_ui, &window);
        imgui_renderer.render(&mut imgui);

        window.gl_swap_window();
    }

    config.set("Imgui", "Show", Some(show_dev_ui.to_string()));
    config.write(CONFIG_FILE).unwrap();
}
