use std::{collections::HashMap, path::PathBuf};

use engine::{
    audio::SoundID,
    geometry::{intersection::point_is_inside_rect, point, Point, Rect},
    Engine,
};

pub struct GameUi {
    cursor: Point,
    cursor_alignment: CursorAlignment,
    buttons: HashMap<String, Button>,
    click_sound: SoundID,
}

#[derive(Debug)]
struct Button {
    is_hot: bool,
    rect: Rect,
    is_hovered: bool,
    is_pressed: bool,
    was_pressed: bool,
    text: String,
}

/// Determines how component will be positioned relative to cursor
enum CursorAlignment {
    TopLeft,
    Centered,
}

pub const BUTTON_WIDTH: u32 = 75;
pub const BUTTON_HEIGHT: u32 = 23;
const SPACING: u32 = 10;

impl Button {
    fn new() -> Self {
        Button {
            is_hot: true,
            rect: Rect {
                x: 0,
                y: 0,
                w: BUTTON_WIDTH,
                h: BUTTON_HEIGHT,
            },
            is_hovered: false,
            is_pressed: false,
            was_pressed: false,
            text: String::new(),
        }
    }
}

impl GameUi {
    pub fn new(engine: &mut Engine) -> Self {
        GameUi {
            cursor: point(0, 0),
            cursor_alignment: CursorAlignment::TopLeft,
            buttons: HashMap::new(),
            click_sound: engine
                .audio_system
                .add_sound(&PathBuf::from("./resources/audio/click.wav")),
        }
    }

    pub fn set_cursor(&mut self, x: i32, y: i32) {
        self.cursor.x = x;
        self.cursor.y = y;
    }

    pub fn button(&mut self, label: &str) -> bool {
        let (id, text) = parse_label(label);
        let button = self.buttons.entry(id.to_string()).or_insert(Button::new());
        let (button_x, button_y) = match self.cursor_alignment {
            CursorAlignment::TopLeft => (self.cursor.x, self.cursor.y),
            CursorAlignment::Centered => (
                self.cursor.x - button.rect.w as i32 / 2,
                self.cursor.y - button.rect.h as i32 / 2,
            ),
        };

        self.cursor += point(0, (BUTTON_HEIGHT + SPACING) as i32);

        button.rect.x = button_x;
        button.rect.y = button_y;
        button.is_hot = true;
        button.text = text.unwrap_or(id).to_owned();

        let released_now = button.is_hovered && button.was_pressed && !button.is_pressed;
        released_now
    }

    pub fn update(&mut self, engine: &Engine) {
        for (_, button) in &mut self.buttons {
            let mouse_intersects_button = point_is_inside_rect(engine.input.mouse.pos, button.rect);
            if mouse_intersects_button && engine.input.mouse.left_button.is_released() {
                button.is_hovered = true;
            } else {
                button.is_hovered = false;
            }

            button.was_pressed = button.is_pressed;
            if mouse_intersects_button && engine.input.mouse.left_button.is_pressed_now() {
                button.is_pressed = true;
                engine.audio_system.play_sound(self.click_sound);
            }
            if !mouse_intersects_button || engine.input.mouse.left_button.is_released() {
                button.is_pressed = false;
            }
        }
    }

    pub fn render(&mut self, engine: &mut Engine) {
        for (_, button) in &self.buttons {
            draw_button(engine, &button);
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

    pub fn draw_centered(&mut self) {
        self.cursor_alignment = CursorAlignment::Centered;
    }

    pub fn _draw_left_aligned(&mut self) {
        self.cursor_alignment = CursorAlignment::TopLeft;
    }
}

#[rustfmt::skip]
fn draw_button(engine: &mut Engine, button: &Button) {
    let rect = button.rect;
    let draw_pressed = button.is_pressed;
    let draw_hovered = button.is_hovered && !button.is_pressed;

    let white = (255, 255, 255);
    let light_grey = (223, 223, 223);
    let hovered_grey = (204, 204, 204);
    let non_hovered_grey = (194, 194, 194);
    let grey = if draw_hovered {
        hovered_grey
    } else {
        non_hovered_grey
    };
    let dark_grey = (129, 129, 129);
    let black = (0, 0, 0);

    let (rect_w, rect_h) = (rect.w as i32, rect.h as i32);

    let top_outline = if draw_pressed { black } else { white };
    let top_highlight = if draw_pressed { dark_grey } else { light_grey };
    let bottom_outline = if draw_pressed { black } else { black };
    let bottom_highlight = if draw_pressed { light_grey } else { dark_grey };

    let renderer = &mut engine.renderer;

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

    // draw text
    if !button.text.is_empty() {
        engine.text_system.set_text_color(0, 0, 0, 255);
        let offset = if draw_pressed { 1 } else { 0 };
        let (text_width, text_height) = engine.text_system.text_dimensions(engine.fonts.arial_16, &button.text);
        let text_x = rect.x + (BUTTON_WIDTH as i32 - text_width as i32) / 2 + offset;
        let text_y = rect.y + (BUTTON_HEIGHT as i32 - text_height as i32) / 2 + offset;
        engine.text_system.draw_text(renderer, engine.fonts.arial_16, text_x, text_y, &button.text);
    }
}

fn parse_label(label: &str) -> (&str, Option<&str>) {
    if let Some((id, text)) = label.split_once("##") {
        (id, Some(text))
    } else {
        (label, None)
    }
}
