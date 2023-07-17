use crate::{graphics::rendering::Canvas, input::button::Button};

pub struct Mouse {
    /// Canvas relative mouse position
    pub pos: glam::IVec2,
    pub scroll_amount: u32,
    pub left_button: Button,
    pub right_button: Button,
    pub middle_button: Button,
    pub x1_button: Button,
    pub x2_button: Button,
    /// Window relative mouse position
    window_pos: glam::IVec2,
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
            window_pos: glam::IVec2::new(0, 0),
        }
    }

    pub fn set_window_pos(&mut self, x: i32, y: i32) {
        self.window_pos = glam::ivec2(x, y);
    }

    pub fn update(&mut self, canvas: &Canvas) {
        self.left_button.update();
        self.right_button.update();
        self.middle_button.update();
        self.x1_button.update();
        self.x2_button.update();

        let offset_x = (self.window_pos.x - canvas.pos.x) as f32;
        let offset_y = (self.window_pos.y - canvas.pos.y) as f32;
        self.pos.x = f32::round(offset_x / canvas.scale) as i32;
        self.pos.y = f32::round(offset_y / canvas.scale) as i32;
    }
}
