pub struct InputDevices {
    pub mouse: Mouse,
}

#[allow(dead_code)]
pub struct Mouse {
    pub pos: glam::IVec2,
    pub scroll_amount: u32,
    pub left_button: Button,
    pub right_button: Button,
    pub middle_button: Button,
    pub x1_button: Button,
    pub x2_button: Button,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Button {
    state: ButtonState,
    event: Option<ButtonEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonEvent {
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ButtonState {
    Released,
    ReleasedNow,
    Pressed,
    PressedNow,
}

impl InputDevices {
    pub fn new() -> Self {
        InputDevices {
            mouse: Mouse::new(),
        }
    }
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            pos: glam::IVec2::new(0, 0),
            scroll_amount: 0,
            left_button: Button::new(),
            right_button: Button::new(),
            middle_button: Button::new(),
            x1_button: Button::new(),
            x2_button: Button::new(),
        }
    }

    pub fn handle_event(&mut self, event: &sdl2::event::Event) {
        use sdl2::mouse::MouseButton;
        match event {
            sdl2::event::Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                MouseButton::Unknown => (),
                MouseButton::Left => {
                    self.left_button.register_event(ButtonEvent::Down);
                }
                MouseButton::Middle => {
                    self.middle_button.register_event(ButtonEvent::Down);
                }
                MouseButton::Right => {
                    self.right_button.register_event(ButtonEvent::Down);
                }
                MouseButton::X1 => {
                    self.x1_button.register_event(ButtonEvent::Down);
                }
                MouseButton::X2 => {
                    self.x2_button.register_event(ButtonEvent::Down);
                }
            },
            sdl2::event::Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                MouseButton::Unknown => (),
                MouseButton::Left => {
                    self.left_button.register_event(ButtonEvent::Up);
                }
                MouseButton::Middle => {
                    self.middle_button.register_event(ButtonEvent::Up);
                }
                MouseButton::Right => {
                    self.right_button.register_event(ButtonEvent::Up);
                }
                MouseButton::X1 => {
                    self.x1_button.register_event(ButtonEvent::Up);
                }
                MouseButton::X2 => {
                    self.x2_button.register_event(ButtonEvent::Up);
                }
            },
            sdl2::event::Event::MouseMotion { x, y, .. } => {
                self.pos = glam::ivec2(*x, *y);
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        self.left_button.update();
        self.right_button.update();
        self.middle_button.update();
        self.x1_button.update();
        self.x2_button.update();
    }
}

#[allow(dead_code)]
impl Button {
    pub fn new() -> Self {
        Button {
            state: ButtonState::Released,
            event: None,
        }
    }

    pub fn register_event(&mut self, event: ButtonEvent) {
        self.event = Some(event);
    }

    pub fn update(&mut self) {
        self.state = self.state.next_state(self.event);
        self.event = None;
    }

    pub fn is_released(&self) -> bool {
        self.state == ButtonState::Released || self.state == ButtonState::ReleasedNow
    }

    pub fn is_pressed(&self) -> bool {
        self.state == ButtonState::Pressed || self.state == ButtonState::PressedNow
    }

    pub fn is_released_now(&self) -> bool {
        self.state == ButtonState::ReleasedNow
    }

    pub fn is_pressed_now(&self) -> bool {
        self.state == ButtonState::PressedNow
    }
}

impl ButtonState {
    fn next_state(&self, event: Option<ButtonEvent>) -> Self {
        match self {
            ButtonState::Released | ButtonState::ReleasedNow => {
                if event == Some(ButtonEvent::Down) {
                    ButtonState::PressedNow
                } else {
                    ButtonState::Released
                }
            }
            ButtonState::Pressed | ButtonState::PressedNow => {
                if event == Some(ButtonEvent::Up) {
                    ButtonState::ReleasedNow
                } else {
                    ButtonState::Pressed
                }
            }
        }
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::Released
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_is_initially_released() {
        let button = Button::new();
        assert!(button.is_released());
    }

    #[test]
    fn button_stays_released_if_no_input() {
        let mut button = Button::new();
        button.update();
        assert!(button.is_released());
    }

    #[test]
    fn button_pressed_if_updated_with_down_event() {
        let mut button = Button::new();

        // initial press
        button.register_event(ButtonEvent::Down);
        button.update();
        assert!(!button.is_released());
        assert!(button.is_pressed());
        assert!(button.is_pressed_now());

        // continue to press
        button.update();
        assert!(!button.is_pressed_now());
        assert!(button.is_pressed());
    }

    #[test]
    fn button_released_if_updated_with_up_event() {
        let mut button = Button::new();

        // initial press
        button.register_event(ButtonEvent::Down);
        button.update();

        // release button
        button.register_event(ButtonEvent::Up);
        button.update();
        assert!(!button.is_pressed());
        assert!(button.is_released_now());
        assert!(button.is_released());

        // continue to hold up button
        button.update();
        assert!(!button.is_released_now());
        assert!(button.is_released());
    }
}
