use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

use crate::input::button::ButtonEvent;

use super::button::Button;

pub struct Keyboard<T> {
    buttons: HashMap<T, Button>,
}

impl<T: PartialEq + Eq + Hash> Keyboard<T> {
    pub fn new() -> Self {
        Keyboard {
            buttons: HashMap::new(),
        }
    }

    pub fn register_event(&mut self, key: T, event: ButtonEvent) {
        self.buttons
            .entry(key)
            .or_insert(Button::default())
            .register_event(event);
    }

    pub fn update(&mut self) {
        for (_, button) in &mut self.buttons {
            button.update();
        }
    }

    #[allow(dead_code)]
    pub fn is_pressed(&self, key: T) -> bool {
        self.buttons
            .get(&key)
            .map(|button| button.is_pressed())
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn is_pressed_now(&self, key: T) -> bool {
        self.buttons
            .get(&key)
            .map(|button| button.is_pressed_now())
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn is_released(&self, key: T) -> bool {
        self.buttons
            .get(&key)
            .map(|button| button.is_released())
            .unwrap_or(true)
    }

    #[allow(dead_code)]
    pub fn is_released_now(&self, key: T) -> bool {
        self.buttons
            .get(&key)
            .map(|button| button.is_released_now())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[parameterized(key = {
        0, 1, 100, u32::MAX
    })]
    fn initially_all_keys_are_up(key: u32) {
        let keyboard = Keyboard::new();

        let is_pressed = keyboard.is_pressed(key);

        assert!(!is_pressed)
    }

    #[parameterized(key1 = {
        0, 1
    }, key2 = {
        100, u32::MAX
    })]
    fn two_buttons_can_be_pressed_simulatenously(key1: u32, key2: u32) {
        let mut keyboard = Keyboard::new();

        keyboard.register_event(key1, ButtonEvent::Down);
        keyboard.register_event(key2, ButtonEvent::Down);
        keyboard.update();

        assert!(keyboard.is_pressed(key1));
        assert!(keyboard.is_pressed(key2));
        assert!(keyboard.is_pressed_now(key1));
        assert!(keyboard.is_pressed_now(key2));
    }

    #[parameterized(key1 = {
        0, 1
    }, key2 = {
        100, u32::MAX
    })]
    fn one_button_can_be_released_while_other_is_held(key1: u32, key2: u32) {
        let mut keyboard = Keyboard::new();

        // press both buttons
        keyboard.register_event(key1, ButtonEvent::Down);
        keyboard.register_event(key2, ButtonEvent::Down);
        keyboard.update();

        // release one of the buttons
        keyboard.register_event(key2, ButtonEvent::Up);
        keyboard.update();

        assert!(keyboard.is_pressed(key1));
        assert!(!keyboard.is_pressed_now(key1));
        assert!(keyboard.is_released(key2));
        assert!(keyboard.is_released_now(key2));
    }
}
