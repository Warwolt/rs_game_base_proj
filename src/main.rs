extern crate freetype;
extern crate gl;
extern crate imgui;
extern crate imgui_opengl_renderer;
extern crate imgui_sdl2;
extern crate sdl2;
#[cfg(test)]
#[macro_use]
extern crate parameterized;

mod audio;
mod geometry;
mod graphics;
mod hot_reload;
mod input;

use crate::audio::AudioPlayer;
use crate::geometry::Dimension;
use crate::graphics::animation::AnimationID;
use crate::graphics::fonts::FontSystem;
use crate::graphics::rendering;
use crate::hot_reload::ResourceReloader;
use crate::input::config::ProgramConfig;
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
use sdl2::event::{Event, WindowEvent};
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
use std::time::SystemTime;

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

fn init_audio(sdl: &sdl2::Sdl) -> sdl2::AudioSubsystem {
    let sdl_audio = sdl.audio().unwrap();
    sdl_audio
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
        // TODO: uncomment this once we have framebuffer render-to-texture and
        // can handle different window resolutions.
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
    renderer.draw_rect_fill(rect);

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
    let init_window_width = 800;
    let init_window_height = 600;

    let window_resolutions = ["200x150", "400x300", "800x600"];
    let mut current_window_resolution = window_resolutions[0];
    let (init_resolution_width, init_resolution_height) = match current_window_resolution {
        "200x150" => (200, 150),
        "400x300" => (400, 300),
        "800x600" => (800, 600),
        _ => panic!("unsupported resolution"),
    };

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

    // init audio
    let _sdl_audio = init_audio(&sdl);
    {
        let frequency = 44_100;
        let format = sdl2::mixer::AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
        let channels = sdl2::mixer::DEFAULT_CHANNELS; // Stereo
        let chunk_size = 1_024;
        sdl2::mixer::open_audio(frequency, format, channels, chunk_size).unwrap();
    }
    let _sdl_mixer = sdl2::mixer::init(sdl2::mixer::InitFlag::MP3).unwrap();

    let window = init_window(
        &sdl_video,
        "Base Project",
        init_window_width,
        init_window_height,
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

    /* Setup audio */
    let mut audio_player = AudioPlayer::new();
    let click_sound_path = PathBuf::from("./resources/audio/click.wav");
    let click_sound = audio_player.add_sound(&click_sound_path);
    let music = audio_player.add_music(&PathBuf::from("./resources/audio/music.wav"));

    /* Setup rendering */
    let mut renderer = Renderer::new(
        init_window_width,
        init_window_height,
        init_resolution_width,
        init_resolution_height,
    );
    let mut sprite_system = SpriteSystem::new();
    let mut animation_system = AnimationSystem::new();
    let mut font_system = FontSystem::new();
    font_system.set_text_color(0, 0, 0, 255);

    /* Main loop */
    let arial_16 = font_system.add_font(
        &mut renderer,
        &PathBuf::from("./resources/font/arial.ttf"),
        16,
    );

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

    let mut smiley_scaling = 2.0;
    let mut smiley_direction = Direction::Down;
    let mut smiley_input_stack = InputStack::<Direction>::new();
    let mut smiley_animatin_is_playing = false;

    let mut button_pressed = false;
    let mut event_pump = sdl.event_pump().unwrap();
    let mut prev_time = SystemTime::now();

    let mut resource_reloader = ResourceReloader::new(&PathBuf::from("./resources"));
    // set up sprite reloading
    {
        let sprite_reloader = &mut resource_reloader.sprite_reloader();

        // register sprites
        sprite_reloader.register_aseprite_sprite_sheet(
            &smiley_image_path,
            &smiley_json_path,
            smiley_texture_id,
            smiley_sprite_sheet_id,
        );

        // register animations
        for (direction, frame_tag_name) in smiley_animation_mappings {
            let animation_id = smiley_animations[&direction];
            sprite_reloader.register_aseprite_animation(
                smiley_json_path,
                animation_id,
                frame_tag_name,
            );
        }
    }
    // set up audio reloading
    {
        let audio_reloader = &mut resource_reloader.audio_reloader();
        audio_reloader.register_sound(click_sound, &click_sound_path);
    }

    // start music
    audio_player.play_music(music);

    'main_loop: loop {
        /* Input */
        let time_now = SystemTime::now();
        let delta_time_ms = time_now.duration_since(prev_time).unwrap().as_millis();
        prev_time = time_now;
        let Dimension {
            width: canvas_width,
            height: canvas_height,
        } = renderer.canvas().dim;

        for event in event_pump.poll_iter() {
            imgui_sdl.handle_event(&mut imgui, &event);
            input.register_event(&event);
            match event {
                Event::Window { win_event, .. } => {
                    if let WindowEvent::Resized(width, height) = win_event {
                        renderer.on_window_resize(width as u32, height as u32)
                    }
                }
                _ => (),
            }
        }
        input.mouse.update(renderer.canvas());
        input.keyboard.update();

        resource_reloader.update(
            delta_time_ms,
            &mut renderer,
            &mut sprite_system,
            &mut animation_system,
            &mut audio_player,
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
            x: (canvas_width - button_width) as i32 / 2,
            y: (canvas_height - button_height) as i32 / 2,
            w: button_width,
            h: button_height,
        };
        let button_was_pressed = button_pressed;
        let mouse_is_inside_button = point_is_inside_rect(input.mouse.pos, button_rect);
        button_pressed = mouse_is_inside_button && input.mouse.left_button.is_pressed();

        // on click
        if !button_pressed && button_was_pressed && mouse_is_inside_button {
            audio_player.play_sound(click_sound);

            if audio_player.music_is_paused() {
                audio_player.resume_music();
            } else {
                audio_player.pause_music();
            }

            smiley_animatin_is_playing = !smiley_animatin_is_playing;
            let animation = smiley_animations[&smiley_direction];
            if smiley_animatin_is_playing {
                animation_system.stop_animation(animation);
            } else {
                animation_system.start_animation(animation);
            }
        }

        // draw dev ui
        imgui_sdl.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());
        let ui = imgui.frame();
        if config.show_dev_ui {
            if let Some(debug_window) = ui.window("Debug Window").begin() {
                ui.text("Hello Win95 Button");
                ui.text(format!(
                    "Mouse pos: ({},{})",
                    input.mouse.pos.x, input.mouse.pos.y
                ));
                ui.text(format!(
                    "Mouse left button pressed: {}",
                    input.mouse.left_button.is_pressed()
                ));
                ui.text(format!("direction = {:?}", smiley_direction));
                ui.slider("Smiley scaling", 1.0, 10.0, &mut smiley_scaling);

                if let Some(_) = ui.begin_combo("Window resolution", current_window_resolution) {
                    for item in &window_resolutions {
                        if ui
                            .selectable_config(item)
                            .selected(current_window_resolution == *item)
                            .build()
                        {
                            current_window_resolution = item;
                            match current_window_resolution {
                                "200x150" => renderer.set_resolution(200, 150),
                                "400x300" => renderer.set_resolution(400, 300),
                                "800x600" => renderer.set_resolution(800, 600),
                                _ => (),
                            }
                        }
                    }
                }

                ui.text(format!("window = {:?}", window.size()));
                ui.text(format!("canvas = {:?}", renderer.canvas().scaled_dim));
                ui.text(format!("scale = {:?}", renderer.canvas().scale));

                debug_window.end();
            };
        }

        /* Render */
        animation_system.update(delta_time_ms as u32);

        // draw background
        renderer.clear();
        renderer.set_draw_color(0, 129, 129, 255);
        renderer.draw_rect_fill(Rect {
            x: 0,
            y: 0,
            w: canvas_width,
            h: canvas_height,
        });

        // draw button
        draw_button(&mut renderer, button_rect, button_pressed);

        // draw button text
        let offset = if button_pressed { 1 } else { 0 };
        let text = if smiley_animatin_is_playing {
            "Play"
        } else {
            "Pause"
        };
        let (text_width, text_height) = font_system.text_dimensions(arial_16, text);
        let text_x = button_rect.x + (button_width as i32 - text_width as i32) / 2 + offset;
        let text_y = button_rect.y + (button_height as i32 - text_height as i32) / 2 + offset;
        font_system.draw_text(&mut renderer, arial_16, text_x, text_y, text);

        // draw smiley
        let smiley_w = 16.0 * smiley_scaling;
        let (smiley_x, smiley_y) = (
            f32::round((canvas_width as f32 - smiley_w) / 2.0) as i32,
            f32::round((canvas_height as f32 - smiley_w) / 2.0) as i32 - 2 * button_height as i32,
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
        imgui_sdl.prepare_render(&ui, &window);
        imgui_renderer.render(&mut imgui);

        window.gl_swap_window();
    }

    config.write_to_disk();
}
