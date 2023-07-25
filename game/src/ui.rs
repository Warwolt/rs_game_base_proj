use std::collections::HashMap;

use engine::{
    geometry::{intersection::point_is_inside_rect, point, Point, Rect},
    graphics::rendering::Renderer,
    input::{self, button::ButtonEvent},
    Engine,
};
pub struct GameUi {
    cursor: Point,
    buttons: HashMap<String, Button>,
}

pub const BUTTON_WIDTH: u32 = 75;
pub const BUTTON_HEIGHT: u32 = 23;
const SPACING: u32 = 10;

#[derive(Debug)]
struct Button {
    is_hot: bool,
    rect: Rect,
    state: input::button::Button,
    text: String,
}

impl Button {
    fn new(x: i32, y: i32) -> Self {
        Button {
            is_hot: true,
            rect: Rect {
                x,
                y,
                w: BUTTON_WIDTH,
                h: BUTTON_HEIGHT,
            },
            state: input::button::Button::new(),
            text: String::new(),
        }
    }
}

impl GameUi {
    pub fn new() -> Self {
        GameUi {
            cursor: point(0, 0),
            buttons: HashMap::new(),
        }
    }

    pub fn set_cursor(&mut self, x: i32, y: i32) {
        self.cursor.x = x;
        self.cursor.y = y;
    }

    // TODO handle the '##' separator with some unit tested parsing
    pub fn button(&mut self, label: &str) -> bool {
        let id = String::from(label);
        let (x, y) = (self.cursor.x, self.cursor.y);
        let button = self.buttons.entry(id).or_insert(Button::new(x, y));
        self.cursor += point(0, (BUTTON_HEIGHT + SPACING) as i32);
        button.is_hot = true;
        button.text = String::from(label);
        button.state.is_pressed_now()
    }

    pub fn update(&mut self, engine: &Engine) {
        for (_, button) in &mut self.buttons {
            let mouse_hover_button = point_is_inside_rect(engine.input().mouse.pos, button.rect);
            let mouse_pressed = engine.input().mouse.left_button.is_pressed();

            let event = if mouse_hover_button && mouse_pressed {
                ButtonEvent::Down
            } else {
                ButtonEvent::Up
            };
            button.state.register_event(event);
            button.state.update();
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        for (_, button) in &self.buttons {
            draw_button(renderer, button.rect, button.state.is_pressed());
        }

        self.remove_cold_components();
    }

    fn remove_cold_components(&mut self) {
        // remove cold components
        self.buttons.retain(|_, button| button.is_hot);

        // mark hot components as cold
        for (_, button) in &mut self.buttons {
            button.is_hot = false;
        }
    }
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
