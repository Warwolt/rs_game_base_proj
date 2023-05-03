use std::{collections::HashMap, hash::Hash, path::Path};

use freetype::face::LoadFlag;

use crate::geometry::Rect;

use super::rendering::{Renderer, TextureID};

pub struct FontSystem {
    library: freetype::Library,
    fonts: HashMap<FontID, HashMap<char, GlyphData>>,
    next_id: u32,
    scaling: f32,
    color: (u8, u8, u8, u8),
    show_bounding_boxes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontID(u32);

#[derive(Debug, Clone, Copy)]
struct GlyphData {
    texture: TextureID,
    width: u32,
    height: u32,
    bearing_x: i32,
    bearing_y: i32,
    advance: u32,
}

impl FontSystem {
    pub fn new() -> Self {
        FontSystem {
            library: freetype::Library::init().unwrap(),
            fonts: HashMap::new(),
            next_id: 0,
            scaling: 1.0,
            color: (255, 255, 255, 255),
            show_bounding_boxes: false,
        }
    }

    pub fn add_font(&mut self, renderer: &mut Renderer, path: &Path, font_size: u32) -> FontID {
        let face = self.library.new_face(path, 0).unwrap();
        face.set_pixel_sizes(0, font_size).unwrap();
        let mut glyphs = HashMap::new();
        let printable_ascii = b' '..=b'~';
        for character in printable_ascii {
            face.load_char(character as usize, LoadFlag::RENDER)
                .unwrap();
            let glyph = face.glyph();
            let bitmap = glyph.bitmap();
            let bitmap_width = bitmap.width() as u32;
            let bitmap_height = bitmap.rows() as u32;
            let mut texture_data = Vec::<u8>::new();

            for y in (0..bitmap_height).rev() {
                for x in 0..bitmap_width {
                    let value = bitmap.buffer()[(x + y * bitmap_width) as usize];
                    texture_data.extend(&[value, value, value, value]);
                }
            }

            let texture_id = renderer.add_texture(&texture_data, bitmap_width, bitmap_height);
            glyphs.insert(
                character as char,
                GlyphData {
                    texture: texture_id,
                    width: bitmap_width,
                    height: bitmap_height,
                    bearing_x: glyph.bitmap_left(),
                    bearing_y: glyph.bitmap_top(),
                    advance: glyph.advance().x as u32,
                },
            );
        }

        let id = self.make_id();
        self.fonts.insert(id, glyphs);

        id
    }

    #[allow(dead_code)]
    pub fn set_scaling(&mut self, scaling: f32) {
        self.scaling = scaling;
    }

    #[allow(dead_code)]
    pub fn set_text_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.color = (r, g, b, a);
    }

    #[allow(dead_code)]
    pub fn show_bounding_boxes(&mut self) {
        self.show_bounding_boxes = true;
    }

    #[allow(dead_code)]
    pub fn hide_bounding_boxes(&mut self) {
        self.show_bounding_boxes = false;
    }

    pub fn draw_text(
        &mut self,
        renderer: &mut Renderer,
        font: FontID,
        mut x: i32,
        mut y: i32,
        text: &str,
    ) {
        let glyphs = &self.fonts[&font];
        let (_, y_max) = self.text_dimensions(font, text);
        y += y_max as i32; // offset the y position by the tallet glyph
        for character in text.chars() {
            assert!(
                character.is_ascii(),
                "non-ascii characters are not yet supported"
            );
            let glyph = glyphs[&character];
            let glyph_x = x + f32::round(glyph.bearing_x as f32 * self.scaling) as i32;
            let glyph_y = y - f32::round(glyph.bearing_y as f32 * self.scaling) as i32;
            let glyph_rect = Rect {
                x: glyph_x,
                y: glyph_y,
                w: f32::round(glyph.width as f32 * self.scaling) as u32,
                h: f32::round(glyph.height as f32 * self.scaling) as u32,
            };
            let (r, g, b, a) = self.color;
            renderer.set_texture_blend_color(r, g, b, a);
            renderer.draw_texture(glyph.texture, glyph_rect, None);

            if self.show_bounding_boxes {
                renderer.set_draw_color(0, 255, 0, 255);
                renderer.draw_rect(glyph_rect);
            }

            x += f32::round(glyph.advance as f32 / 64.0 * self.scaling) as i32;
        }
    }

    pub fn text_dimensions(&self, font: FontID, text: &str) -> (u32, u32) {
        let glyphs = &self.fonts[&font];

        let mut width = 0;
        let mut height = 0;
        for character in text.chars() {
            let glyph = glyphs[&character];
            let scaled_height = f32::round(glyph.height as f32 * self.scaling) as u32;
            let scaled_advance = f32::round(glyph.advance as f32 / 64.0 * self.scaling) as u32;

            width += scaled_advance;
            height = u32::max(height, scaled_height);
        }

        (width, height)
    }

    fn make_id(&mut self) -> FontID {
        let id = self.next_id;
        self.next_id += 1;
        FontID(id)
    }
}
