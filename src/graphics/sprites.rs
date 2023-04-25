use std::collections::HashMap;

use crate::{
    geometry::Rect,
    graphics::rendering::{Renderer, TextureData},
};

use super::rendering::texture_from_image_path;

#[derive(Debug)]
pub struct SpriteSystem {
    sprite_sheets: HashMap<SpriteSheetID, SpriteSheetData>,
    next_id: u32,
    scaling: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteSheetID(u32);

#[derive(Debug, Clone)]
struct SpriteSheetData {
    texture: TextureData,
    sprites: Vec<Rect>,
    color_key: Option<(u8, u8, u8)>,
}

impl SpriteSystem {
    pub fn new() -> Self {
        SpriteSystem {
            sprite_sheets: HashMap::new(),
            next_id: 0,
            scaling: 1.0,
        }
    }

    pub fn add_spritesheet(
        &mut self,
        texture: &TextureData,
        sprites: &[Rect],
        color_key: Option<(u8, u8, u8)>,
    ) -> SpriteSheetID {
        let id = self.make_id();
        let sprite_sheet = SpriteSheetData {
            texture: *texture,
            sprites: Vec::from(sprites),
            color_key,
        };
        self.sprite_sheets.insert(id, sprite_sheet);
        id
    }

    #[allow(dead_code)]
    pub fn spritesheet_dimensions(&self, sprite_sheet: SpriteSheetID) -> (u32, u32) {
        let sprite_sheet = &self.sprite_sheets[&sprite_sheet];
        (sprite_sheet.texture.width, sprite_sheet.texture.height)
    }

    pub fn set_scaling(&mut self, scaling: f32) {
        self.scaling = scaling;
    }

    pub fn reset_scaling(&mut self) {
        self.scaling = 1.0;
    }

    pub fn draw_sprite(
        &self,
        renderer: &mut Renderer,
        sprite_sheet: SpriteSheetID,
        sprite_index: usize,
        x: i32,
        y: i32,
    ) {
        let sprite_sheet = &self.sprite_sheets[&sprite_sheet];
        let sprite_rect = sprite_sheet.sprites[sprite_index];

        if let Some((r, g, b)) = sprite_sheet.color_key {
            renderer.set_color_key(r, g, b);
        }

        renderer.draw_texture(
            sprite_sheet.texture.id,
            Rect {
                x,
                y,
                w: f32::round(sprite_rect.w as f32 * self.scaling) as u32,
                h: f32::round(sprite_rect.h as f32 * self.scaling) as u32,
            },
            Some(sprite_rect),
        );

        renderer.disable_color_key();
    }

    fn make_id(&mut self) -> SpriteSheetID {
        let id = self.next_id;
        self.next_id += 1;
        SpriteSheetID(id)
    }
}

pub fn load_aseprite_spritesheet(
    sprite_system: &mut SpriteSystem,
    renderer: &mut Renderer,
    image_path: &str,
    json_path: &str,
) -> (SpriteSheetID, aseprite::SpritesheetData) {
    let json_file = std::fs::File::open(json_path).unwrap();
    let sprite_sheet_data: aseprite::SpritesheetData = serde_json::from_reader(json_file).unwrap();
    let texture = texture_from_image_path(renderer, image_path);
    let frames = sprite_sheet_data
        .frames
        .iter()
        .map(|frame| Rect {
            x: frame.frame.x as i32,
            y: frame.frame.y as i32,
            w: frame.frame.w,
            h: frame.frame.h,
        })
        .collect::<Vec<Rect>>();

    let id = sprite_system.add_spritesheet(&texture, &frames, None);

    (id, sprite_sheet_data)
}
