extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;
#[cfg(test)]
#[macro_use]
extern crate parameterized;

mod geometry;
mod graphics;
mod hot_reload;
mod input;

use crate::graphics::animation::AnimationID;
use crate::graphics::rendering;
use crate::hot_reload::AsepriteReloader;
use crate::input::config::ProgramConfig;
use crate::input::file::FileWatcher;
use crate::{
    graphics::{
        animation::{self, AnimationSystem},
        sprites,
    },
    input::input_stack::InputStack,
};
use geometry::Rect;
use graphics::{rendering::Renderer, sprites::SpriteSystem};
use input::InputDevices;
use sdl2::{
    keyboard::Keycode,
    video::{GLContext, GLProfile, Window},
    VideoSubsystem,
};
use simple_logger::SimpleLogger;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::str;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Right,
    Up,
    Left,
    Down,
}

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

fn init_window(
    sdl_video: &VideoSubsystem,
    title: &str,
    width: u32,
    height: u32,
    monitor: i32,
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

    if let Ok(bounds) = sdl_video.display_bounds(monitor) {
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

// TODO move this to geometry?
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

fn main() {
    let window_width = 800;
    let window_height = 600;

    /* Initialize logging */
    init_logging();
    log::info!("Program start");

    /* Initialize configuration */
    let mut config = ProgramConfig::from_file(&PathBuf::from("config.ini"));

    /* Parse args */
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && args[1] == "--monitor" {
        config.monitor = args[2].parse::<u64>().unwrap();
    }

    /* Initialize SDL */
    let sdl = sdl2::init().unwrap();
    let sdl_video = init_video(&sdl);
    let window = init_window(
        &sdl_video,
        "Base Project",
        window_width,
        window_height,
        config.monitor as i32,
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
    let mut renderer = Renderer::new();
    renderer.on_window_resize(window_width, window_height); // FIXME make width height args to new()
    let mut sprite_system = SpriteSystem::new();
    let mut animation_system = AnimationSystem::new();

    /* Main loop */
    let smiley_image_path = PathBuf::from(r"resources/smiley.png");
    let smiley_json_path = &PathBuf::from(r"resources/smiley.json");
    let smiley_texture_id =
        rendering::load_texture_from_image_path(&mut renderer, &smiley_image_path).unwrap();
    let mut smiley_animations = HashMap::<Direction, AnimationID>::new();
    let smiley_animation_mappings = [
        (Direction::Right, "Right"),
        (Direction::Up, "Up"),
        (Direction::Left, "Left"),
        (Direction::Down, "Down"),
    ];
    let smiley_sprite_sheet_id;
    {
        let sprite_sheet_data = sprites::load_aseprite_sprite_sheet(&smiley_json_path).unwrap();
        let frames = sprites::aseprite_sprite_sheet_frames(&sprite_sheet_data);
        smiley_sprite_sheet_id = sprite_system.add_spritesheet(smiley_texture_id, &frames, None);

        for (direction, frame_tag_name) in smiley_animation_mappings {
            smiley_animations.entry(direction).or_insert(
                animation::add_asperite_sprite_sheet_animation(
                    &mut animation_system,
                    &sprite_sheet_data,
                    frame_tag_name,
                ),
            );
            animation_system.start_animation(smiley_animations[&direction]);
        }
    };
    let mut smiley_scaling = 5.0;
    let mut smiley_direction = Direction::Down;
    let mut smiley_input_stack = InputStack::<Direction>::new();

    let mut button_pressed = false;
    let mut event_pump = sdl.event_pump().unwrap();
    let mut prev_time = SystemTime::now();

    // set up hot reloading
    let file_watcher_debounce = Duration::from_millis(1000);
    let mut file_watcher = FileWatcher::new(&PathBuf::from("./resources"), file_watcher_debounce);
    let mut aseprite_reloader = AsepriteReloader::new();
    {
        // register sprites
        aseprite_reloader.register_aseprite_sprite_sheet(
            &smiley_image_path,
            &smiley_json_path,
            smiley_texture_id,
            smiley_sprite_sheet_id,
        );

        // register animations
        for (direction, frame_tag_name) in smiley_animation_mappings {
            let animation_id = smiley_animations[&direction];
            aseprite_reloader.register_aseprite_animation(
                smiley_json_path,
                animation_id,
                frame_tag_name,
            );
        }
    }

    'main_loop: loop {
        /* Input */
        let time_now = SystemTime::now();
        let delta_time_ms = time_now.duration_since(prev_time).unwrap().as_millis();
        prev_time = time_now;

        for event in event_pump.poll_iter() {
            imgui_sdl.handle_event(&mut imgui, &event);
            input.register_event(&event);
        }
        input.update();

        /* Update */
        let updated_resource_files = file_watcher.update(delta_time_ms);
        aseprite_reloader.update(
            &mut renderer,
            &mut sprite_system,
            &mut animation_system,
            &updated_resource_files,
        );

        if input.keyboard.is_pressed_now(Keycode::Escape) || input.quit {
            break 'main_loop;
        }
        if input.keyboard.is_pressed_now(Keycode::F3) {
            config.show_dev_ui = !config.show_dev_ui;
        }

        // update smiley direction
        let smiley_input_mappings = [
            (Keycode::Right, Direction::Right),
            (Keycode::Up, Direction::Up),
            (Keycode::Left, Direction::Left),
            (Keycode::Down, Direction::Down),
        ];
        for (keycode, direction) in smiley_input_mappings {
            if input.keyboard.is_pressed_now(keycode) {
                smiley_input_stack.push(direction);
            }
            if input.keyboard.is_released_now(keycode) {
                smiley_input_stack.remove(&direction);
            }
        }
        smiley_direction = *smiley_input_stack.top().unwrap_or(&smiley_direction);

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
            animation_system.reset_animation(smiley_animations[&smiley_direction]);
        }

        // draw dev ui
        imgui_sdl.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let dev_ui = imgui.frame();
        if config.show_dev_ui {
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
                dev_ui.text(format!("direction = {:?}", smiley_direction));
                dev_ui.slider("Smiley scaling", 0.1, 10.0, &mut smiley_scaling);
                if dev_ui.button("reset scaling") {
                    smiley_scaling = 1.0;
                }

                window.end();
            };
        }

        /* Render */
        animation_system.update(delta_time_ms as u32);

        // draw background
        renderer.clear();
        renderer.set_draw_color(0, 129, 129, 255);
        renderer.draw_rect_fill(0, 0, window_width as i32, window_height as i32);

        // draw button
        draw_button(&mut renderer, button_rect, button_pressed);

        // draw smiley
        let smiley_w = 16.0 * smiley_scaling;
        let (smiley_x, smiley_y) = (
            f32::round((window_width as f32 - smiley_w) / 2.0) as i32,
            f32::round((window_height as f32 - smiley_w) / 2.0) as i32 - 100,
        );
        let smiley_frame = animation_system.current_frame(smiley_animations[&smiley_direction]);
        sprite_system.set_scaling(smiley_scaling);
        sprite_system.draw_sprite(
            &mut renderer,
            smiley_sprite_sheet_id,
            smiley_frame,
            smiley_x,
            smiley_y,
        );
        sprite_system.reset_scaling();

        renderer.render();
        imgui_sdl.prepare_render(&dev_ui, &window);
        imgui_renderer.render(&mut imgui);

        window.gl_swap_window();
    }

    config.write_to_disk();
}
