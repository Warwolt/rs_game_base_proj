use sdl2::{
    sys::SDL_WindowFlags,
    video::{Window, WindowPos},
    VideoSubsystem,
};

use crate::geometry::Rect;

pub struct FullscreenSystem {
    last_windowed_pos: (i32, i32),
    last_windowed_size: (u32, u32),
}

#[derive(PartialEq)]
enum WindowMode {
    Windowed,
    Fullscreen,
}

impl FullscreenSystem {
    pub fn new() -> Self {
        FullscreenSystem {
            last_windowed_pos: (0, 0),
            last_windowed_size: (0, 0),
        }
    }

    pub fn update(&mut self, window: &Window) {
        if window_mode(window) == WindowMode::Windowed {
            self.last_windowed_pos = window.position();
            self.last_windowed_size = window.size();
        }
    }

    pub fn toggle_fullscreen(&self, window: &mut Window, sdl_video: &VideoSubsystem) {
        match window_mode(window) {
            WindowMode::Windowed => {
                change_to_fullscreen_mode(window, sdl_video);
            }
            WindowMode::Fullscreen => {
                change_to_windowed_mode(window, self.last_windowed_pos, self.last_windowed_size);
            }
        }
    }
}

fn window_mode(window: &Window) -> WindowMode {
    if window.window_flags() & SDL_WindowFlags::SDL_WINDOW_BORDERLESS as u32 == 0 {
        WindowMode::Windowed
    } else {
        WindowMode::Fullscreen
    }
}

fn change_to_fullscreen_mode(window: &mut Window, sdl_video: &VideoSubsystem) {
    let Rect {
        x,
        y,
        w: width,
        h: height,
    } = screen_rect(window, sdl_video);
    window.set_position(WindowPos::Positioned(x), WindowPos::Positioned(y));
    window.set_bordered(false);
    window.set_size(width, height).unwrap();
}

fn change_to_windowed_mode(
    window: &mut Window,
    last_windowed_pos: (i32, i32),
    last_windowed_size: (u32, u32),
) {
    window.set_bordered(true);
    window.set_position(
        WindowPos::Positioned(last_windowed_pos.0),
        WindowPos::Positioned(last_windowed_pos.1),
    );
    window
        .set_size(last_windowed_size.0, last_windowed_size.1)
        .unwrap();
}

fn screen_rect(window: &Window, sdl_video: &VideoSubsystem) -> Rect {
    let display_index = window.display_index().unwrap();
    let display_mode = sdl_video.desktop_display_mode(display_index).unwrap();
    let num_displays = sdl_video.num_video_displays().unwrap();
    let mut display_bounds = Vec::new();
    for i in 0..num_displays {
        display_bounds.push(sdl_video.display_bounds(i).unwrap());
    }
    let x: i32 = display_bounds.iter().map(|rect| rect.top_left().x).sum();
    let y: i32 = display_bounds[display_index as usize].top();

    Rect {
        x,
        y,
        w: display_mode.w as u32,
        h: display_mode.h as u32,
    }
}
