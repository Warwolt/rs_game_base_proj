extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;

mod geometry;
mod input;
mod midpoint;
mod rendering;

use std::path::Path;
use std::str;

use configparser::ini::Ini;
use image::GenericImageView;
use input::InputDevices;
use rendering::{Renderer, TextureData};
use sdl2::keyboard::Keycode;
use sdl2::video::GLContext;
use sdl2::VideoSubsystem;
use sdl2::{
    event::Event,
    video::{GLProfile, Window},
};
use simple_logger::SimpleLogger;
use std::time::SystemTime;

use crate::geometry::Rect;

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

    let debug_gl = true;
    if debug_gl {
        gl_attr.set_context_flags().debug().set();
    }

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

    let (rect_w, rect_h) = (rect.w as i32, rect.h as i32);

    let top_outline = if pressed { black } else { white };
    let top_highlight = if pressed { dark_grey } else { light_grey };
    let bottom_outline = if pressed { black } else { black };
    let bottom_highlight = if pressed { light_grey } else { dark_grey };

    // button body
    renderer.set_draw_color(grey.0, grey.1, grey.2, 255);
    renderer.draw_rect_fill(rect.x, rect.y, rect_w, rect_h);

    // top outline
    renderer.set_draw_color(top_outline.0, top_outline.1, top_outline.2, 255);
    renderer.draw_line(rect.x, rect.y, rect.x, rect.y + rect_h);
    renderer.draw_line(rect.x, rect.y, rect.x + rect_w, rect.y);

    // top highlight
    renderer.set_draw_color(top_highlight.0, top_highlight.1, top_highlight.2, 255);
    renderer.draw_line(rect.x + 1, rect.y + 1, rect.x + 1, rect.y + rect_h - 1);
    renderer.draw_line(rect.x + 1, rect.y + 1, rect.x + rect_w - 1, rect.y);

    // bottom outline
    renderer.set_draw_color(bottom_outline.0, bottom_outline.1, bottom_outline.2, 255);
    renderer.draw_line(rect.x + rect_w, rect.y, rect.x + rect_w, rect.y + rect_h);
    renderer.draw_line(rect.x, rect.y + rect_h, rect.x + rect_w, rect.y + rect_h);

    // bottom highlight
    renderer.set_draw_color(bottom_highlight.0, bottom_highlight.1, bottom_highlight.2, 255);
    renderer.draw_line(rect.x + rect_w - 1, rect.y + 1, rect.x + rect_w - 1, rect.y + rect_h - 1);
    renderer.draw_line(rect.x + 1, rect.y + rect_h - 1, rect.x + rect_w - 1, rect.y + rect_h - 1);
}

fn point_is_inside_rect(point: glam::IVec2, rect: Rect) -> bool {
    let rect_x0 = rect.x;
    let rect_y0 = rect.y;
    let rect_x1 = rect.x + rect.w as i32;
    let rect_y1 = rect.y + rect.h as i32;
    let (point_x, point_y) = (point.x as i32, point.y as i32);

    let horizontal_overlap = rect_x0 <= point_x && point_x <= rect_x1;
    let vertical_overlap = rect_y0 <= point_y && point_y <= rect_y1;

    horizontal_overlap && vertical_overlap
}

fn texture_from_image_path(renderer: &mut Renderer, path: &str) -> TextureData {
    let image = image::open(path).unwrap().flipv();
    let (width, height) = image.dimensions();
    let data = image
        .pixels()
        .flat_map(|(_x, _y, pixel)| pixel.0)
        .collect::<Vec<u8>>();
    let id = renderer.add_texture(&data, width, height);

    log::info!("Loaded image \"{}\"", path);

    TextureData { id, width, height }
}

fn main() {
    let window_width = 800;
    let window_height = 600;

    /* Initialize logging */
    init_logging();
    log::info!("Program start");

    /* Initialize configuration */
    // TODO: setup a way of serializing/deserializing our own config struct
    // instead of working directly with this `Ini` value
    let mut config = Ini::new();
    if Path::new("./config.ini").exists() {
        // loading existing file
        config.load("config.ini").unwrap();
        log::info!("Read config file");
    } else {
        // set defaults on new file
        config.set("Imgui", "Show", Some(String::from("false")));
        log::info!("No existing config file, new one will be created");
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
    log::info!("SDL initialized");

    /* Initialize OpenGL */
    let _gl_context = init_opengl(&window); // closes on drop
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as _);
    log::info!("Created OpenGL context");

    /* Initialize ImGui */
    let mut imgui = imgui::Context::create();
    let mut imgui_sdl = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);
    let get_proc_address = |s| sdl_video.gl_get_proc_address(s) as _;
    let imgui_renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, get_proc_address);
    log::info!("ImGui initialied");

    /* Setup input */
    let mut input = InputDevices::new();

    /* Setup rendering */
    let mut renderer = rendering::Renderer::new();
    renderer.on_window_resize(window_width, window_height);

    /* Main loop */
    let smiley_texture = texture_from_image_path(&mut renderer, "resources/smiley.png");
    let spritesheet_texture = texture_from_image_path(&mut renderer, "resources/spritesheet.png");
    let sprite_rects: [Rect; 4] = [
        // "1"
        Rect {
            x: 0,
            y: 0,
            w: 32,
            h: 32,
        },
        // "2"
        Rect {
            x: 32,
            y: 0,
            w: 32,
            h: 32,
        },
        // "3"
        Rect {
            x: 0,
            y: 32,
            w: 32,
            h: 32,
        },
        // "4"
        Rect {
            x: 32,
            y: 32,
            w: 32,
            h: 32,
        },
    ];
    let smiley_center = (
        (window_width - smiley_texture.width) as i32 / 2,
        (window_height - smiley_texture.height) as i32 / 2,
    );
    let smiley_positions: [(i32, i32); 4] = [
        // right
        (smiley_center.0 + 100, smiley_center.1),
        // top
        (smiley_center.0, smiley_center.1 - 100),
        // left
        (smiley_center.0 - 100, smiley_center.1),
        // down
        (smiley_center.0, smiley_center.1 + 100),
    ];
    let mut current_sprite = 0;
    let mut button_pressed = false;

    let mut event_pump = sdl.event_pump().unwrap();
    let mut prev_time = SystemTime::now();
    let mut show_dev_ui = config.getbool("Imgui", "Show").unwrap().unwrap();
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
            x: (window_width - button_width) as i32 / 2,
            y: (window_height - button_height) as i32 / 2,
            w: button_width,
            h: button_height,
        };

        let button_was_pressed = button_pressed;
        let mouse_is_inside_button = point_is_inside_rect(input.mouse.pos, button_rect);
        button_pressed = mouse_is_inside_button && input.mouse.left_button.is_pressed();

        // on click
        if !button_pressed && button_was_pressed && mouse_is_inside_button {
            current_sprite = (current_sprite + 1) % sprite_rects.len();
        }

        draw_button(&mut renderer, button_rect, button_pressed);

        renderer.set_color_key(255, 0, 255);
        let (smiley_x, smile_y) = smiley_positions[current_sprite];
        renderer.draw_texture(
            smiley_texture.id,
            Rect {
                x: smiley_x,
                y: smile_y,
                w: smiley_texture.width,
                h: smiley_texture.height,
            },
            None,
        );
        let pressed_offset = if button_pressed { (1, 1) } else { (0, 0) };
        renderer.draw_texture(
            spritesheet_texture.id,
            Rect {
                x: button_rect.x + 20 + pressed_offset.0,
                y: button_rect.y - 4 + pressed_offset.1,
                w: 32,
                h: 32,
            },
            Some(sprite_rects[current_sprite]),
        );
        renderer.disable_color_key();

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
