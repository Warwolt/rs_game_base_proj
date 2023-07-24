#[cfg(test)]
#[macro_use]
extern crate parameterized;

pub mod audio;
pub mod geometry;
pub mod graphics;
pub mod hot_reload;
pub mod imgui;
pub mod input;

use crate::input::config::ProgramConfig;
use sdl2::keyboard::Keycode;

use crate::{
    audio::AudioSystem,
    graphics::{
        animation::AnimationSystem, fonts::FontSystem, fullscreen::FullscreenSystem,
        rendering::Renderer, sprites::SpriteSystem,
    },
    input::InputDevices,
};
use itertools::Itertools;
use sdl2::video::GLProfile;
use std::time::SystemTime;

pub struct Engine<'a> {
    // SDL
    _sdl: sdl2::Sdl,
    sdl_video: sdl2::VideoSubsystem,
    _sdl_audio: sdl2::AudioSubsystem,
    _sdl_mixer: sdl2::mixer::Sdl2MixerContext,
    sdl_event_pump: sdl2::EventPump,
    _gl_context: sdl2::video::GLContext,

    // Game Loop
    window: sdl2::video::Window,
    input: InputDevices,
    renderer: Renderer,
    frame: FrameTime,
    should_quit: bool,

    // Systems
    pub fullscreen_system: FullscreenSystem,
    _audio_system: AudioSystem<'a>,
    _sprite_system: SpriteSystem,
    _animation_system: AnimationSystem,
    _font_system: FontSystem,
}

pub struct FrameTime {
    pub delta_ms: u128,
    prev_time: SystemTime,
}

pub fn init<'a>(config: &ProgramConfig, window_width: u32, window_height: u32) -> Engine<'a> {
    // SDL
    let sdl = sdl2::init().unwrap();
    let sdl_video = init_video(&sdl);
    let sdl_audio = init_audio(&sdl);
    let sdl_mixer = init_mixer(&sdl_audio);
    let sdl_event_pump = sdl.event_pump().unwrap();
    let window = init_window(
        &sdl_video,
        "Game",
        window_width,
        window_height,
        config.monitor as i32,
    );
    log::info!("SDL initialized");

    // OpenGL
    let _gl_context = init_opengl(&window); // closes on drop
    gl::load_with(|s| sdl_video.gl_get_proc_address(s) as _);
    log::info!("Created OpenGL context");

    // Game Loop
    let input = InputDevices::new();
    let renderer = Renderer::new(window_width, window_height);
    let frame = FrameTime {
        delta_ms: 0,
        prev_time: SystemTime::now(),
    };
    let should_quit = false;

    // Systems
    let fullscreen_system = FullscreenSystem::new();
    let audio_system = AudioSystem::new();
    let sprite_system = SpriteSystem::new();
    let animation_system = AnimationSystem::new();
    let font_system = FontSystem::new();

    Engine {
        // SDL
        _sdl: sdl,
        sdl_video,
        _sdl_audio: sdl_audio,
        _sdl_mixer: sdl_mixer,
        sdl_event_pump,
        _gl_context,

        // Game Loop
        window,
        input,
        renderer,
        frame,
        should_quit,

        // Systems
        fullscreen_system,
        _audio_system: audio_system,
        _sprite_system: sprite_system,
        _animation_system: animation_system,
        _font_system: font_system,
    }
}

impl<'a> Engine<'a> {
    pub fn begin_frame(&mut self) -> Vec<sdl2::event::Event> {
        let time_now = SystemTime::now();
        self.frame.delta_ms = time_now
            .duration_since(self.frame.prev_time)
            .unwrap()
            .as_millis();
        self.frame.prev_time = time_now;
        self.sdl_event_pump.poll_iter().collect_vec()
    }

    pub fn input(&self) -> &InputDevices {
        &self.input
    }

    pub fn renderer(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    pub fn handle_input(&mut self, events: &Vec<sdl2::event::Event>) {
        for event in events {
            self.input.register_event(&event);
            match *event {
                sdl2::event::Event::Window { win_event, .. } => {
                    if let sdl2::event::WindowEvent::Resized(width, height) = win_event {
                        self.renderer.on_window_resize(width as u32, height as u32)
                    }
                }
                _ => (),
            }
        }
        self.input.mouse.update(self.renderer.canvas());
        self.input.keyboard.update();
    }

    pub fn update(&mut self) {
        self.fullscreen_system.update(&self.window);
        if self.input.keyboard.is_pressed_now(Keycode::F11) {
            self.fullscreen_system
                .toggle_fullscreen(&mut self.window, &self.sdl_video);
            let (width, height) = self.window.size();
            self.renderer.on_window_resize(width, height);
        }

        if self.input.keyboard.is_pressed_now(Keycode::Escape) || self.input.quit {
            self.should_quit = true;
        }
    }

    pub fn render(&mut self) {
        self.renderer.render();
    }

    pub fn end_frame(&mut self) {
        self.window.gl_swap_window();
    }
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

fn init_mixer(_sdl_audio: &sdl2::AudioSubsystem) -> sdl2::mixer::Sdl2MixerContext {
    let frequency = 44_100;
    let format = sdl2::mixer::AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = sdl2::mixer::DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size).unwrap();
    sdl2::mixer::init(sdl2::mixer::InitFlag::MP3).unwrap()
}

fn init_window(
    sdl_video: &sdl2::VideoSubsystem,
    title: &str,
    width: u32,
    height: u32,
    monitor: i32,
) -> sdl2::video::Window {
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

fn init_opengl(window: &sdl2::video::Window) -> sdl2::video::GLContext {
    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();
    window.subsystem().gl_set_swap_interval(1).unwrap();
    return gl_context;
}
