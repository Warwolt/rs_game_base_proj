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
use graphics::fonts::FontID;
use sdl2::{keyboard::Keycode, video::GLContext};

use crate::{
    audio::AudioSystem,
    graphics::{
        animation::AnimationSystem, fonts::TextSystem, fullscreen::FullscreenSystem,
        rendering::Renderer, sprites::SpriteSystem,
    },
    input::InputDevices,
};
use itertools::Itertools;
use sdl2::video::GLProfile;
use std::{path::PathBuf, time::SystemTime};

pub struct Engine<'a> {
    // SDL
    _sdl: sdl2::Sdl,
    sdl_video: sdl2::VideoSubsystem,
    _sdl_audio: sdl2::AudioSubsystem,
    _sdl_mixer: sdl2::mixer::Sdl2MixerContext,
    sdl_event_pump: sdl2::EventPump,

    // Game Loop
    pub window: sdl2::video::Window,
    pub input: InputDevices,
    pub renderer: Renderer,
    pub frame: FrameTime,
    should_quit: bool,

    // Systems
    pub fullscreen_system: FullscreenSystem,
    _audio_system: AudioSystem<'a>,
    _sprite_system: SpriteSystem,
    _animation_system: AnimationSystem,
    pub text_system: TextSystem,

    // Assets
    pub fonts: LoadedFonts,
}

pub struct SdlContext {
    pub sdl: sdl2::Sdl,
    pub sdl_video: sdl2::VideoSubsystem,
    pub sdl_audio: sdl2::AudioSubsystem,
    pub sdl_mixer: sdl2::mixer::Sdl2MixerContext,
    pub sdl_event_pump: sdl2::EventPump,
    pub window: sdl2::video::Window,
}

pub struct FrameTime {
    pub delta_ms: u128,
    prev_time: SystemTime,
}

pub struct LoadedFonts {
    pub arial_16: FontID,
}

pub fn init_sdl(config: &ProgramConfig, window_width: u32, window_height: u32) -> SdlContext {
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

    SdlContext {
        sdl,
        sdl_video,
        sdl_audio,
        sdl_mixer,
        sdl_event_pump,
        window,
    }
}

pub fn init_opengl(sdl: &SdlContext) -> GLContext {
    let gl_context = sdl.window.gl_create_context().unwrap();
    sdl.window.gl_make_current(&gl_context).unwrap();
    sdl.window.subsystem().gl_set_swap_interval(1).unwrap();
    gl::load_with(|s| sdl.sdl_video.gl_get_proc_address(s) as _);
    log::info!("Created OpenGL context");
    return gl_context;
}

pub fn init_engine<'a>(sdl: SdlContext, gl: &GLContext) -> Engine<'a> {
    // Game Loop
    let (window_width, window_height) = sdl.window.size();
    let input = InputDevices::new();
    let mut renderer = Renderer::new(gl, window_width, window_height);
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
    let mut text_system = TextSystem::new();

    // Assets
    let arial_16 = text_system.load_font(
        gl,
        &mut renderer,
        &PathBuf::from("./resources/font/arial.ttf"),
        16,
    );

    let fonts = LoadedFonts { arial_16 };

    Engine {
        // SDL
        _sdl: sdl.sdl,
        sdl_video: sdl.sdl_video,
        _sdl_audio: sdl.sdl_audio,
        _sdl_mixer: sdl.sdl_mixer,
        sdl_event_pump: sdl.sdl_event_pump,

        // Game Loop
        window: sdl.window,
        input,
        renderer,
        frame,
        should_quit,

        // Systems
        fullscreen_system,
        _audio_system: audio_system,
        _sprite_system: sprite_system,
        _animation_system: animation_system,
        text_system: text_system,

        // Assets
        fonts,
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

    pub fn render(&mut self, gl: &GLContext) {
        self.renderer.render(gl);
    }

    pub fn end_frame(&mut self, _gl: &GLContext) {
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
