pub mod button;
pub mod file;
pub mod input_stack;
pub mod keyboard;
pub mod mouse;

use crate::input::button::ButtonEvent;

use self::{keyboard::Keyboard, mouse::Mouse};

pub struct InputDevices {
    pub quit: bool,
    pub mouse: Mouse,
    pub keyboard: Keyboard<sdl2::keyboard::Keycode>,
}

impl InputDevices {
    pub fn new() -> Self {
        InputDevices {
            quit: false,
            mouse: Mouse::new(),
            keyboard: Keyboard::new(),
        }
    }

    pub fn register_event(&mut self, event: &sdl2::event::Event) {
        use sdl2::mouse::MouseButton;
        match event {
            sdl2::event::Event::Quit { .. } => {
                self.quit = true;
            }
            sdl2::event::Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                MouseButton::Left => {
                    self.mouse.left_button.register_event(ButtonEvent::Down);
                }
                MouseButton::Middle => {
                    self.mouse.middle_button.register_event(ButtonEvent::Down);
                }
                MouseButton::Right => {
                    self.mouse.right_button.register_event(ButtonEvent::Down);
                }
                MouseButton::X1 => {
                    self.mouse.x1_button.register_event(ButtonEvent::Down);
                }
                MouseButton::X2 => {
                    self.mouse.x2_button.register_event(ButtonEvent::Down);
                }
                _ => (),
            },
            sdl2::event::Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                MouseButton::Left => {
                    self.mouse.left_button.register_event(ButtonEvent::Up);
                }
                MouseButton::Middle => {
                    self.mouse.middle_button.register_event(ButtonEvent::Up);
                }
                MouseButton::Right => {
                    self.mouse.right_button.register_event(ButtonEvent::Up);
                }
                MouseButton::X1 => {
                    self.mouse.x1_button.register_event(ButtonEvent::Up);
                }
                MouseButton::X2 => {
                    self.mouse.x2_button.register_event(ButtonEvent::Up);
                }
                _ => (),
            },
            sdl2::event::Event::MouseMotion { x, y, .. } => {
                self.mouse.pos = glam::ivec2(*x, *y);
            }
            sdl2::event::Event::KeyDown { keycode, .. } => {
                if let Some(keycode) = keycode {
                    self.keyboard.register_event(*keycode, ButtonEvent::Down)
                }
            }
            sdl2::event::Event::KeyUp { keycode, .. } => {
                if let Some(keycode) = keycode {
                    self.keyboard.register_event(*keycode, ButtonEvent::Up)
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.mouse.update();
        self.keyboard.update();
    }
}
