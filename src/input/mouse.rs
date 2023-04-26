use crate::input::button::Button;

pub struct Mouse {
    pub pos: glam::IVec2,
    pub scroll_amount: u32,
    pub left_button: Button,
    pub right_button: Button,
    pub middle_button: Button,
    pub x1_button: Button,
    pub x2_button: Button,
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

    pub fn update(&mut self) {
        self.left_button.update();
        self.right_button.update();
        self.middle_button.update();
        self.x1_button.update();
        self.x2_button.update();
    }
}
