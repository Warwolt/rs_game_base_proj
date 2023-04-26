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
pub enum ButtonState {
    Released,
    ReleasedNow,
    Pressed,
    PressedNow,
}

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

    #[allow(dead_code)]
    pub fn is_released(&self) -> bool {
        self.state == ButtonState::Released || self.state == ButtonState::ReleasedNow
    }

    #[allow(dead_code)]
    pub fn is_pressed(&self) -> bool {
        self.state == ButtonState::Pressed || self.state == ButtonState::PressedNow
    }

    #[allow(dead_code)]
    pub fn is_released_now(&self) -> bool {
        self.state == ButtonState::ReleasedNow
    }

    #[allow(dead_code)]
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
