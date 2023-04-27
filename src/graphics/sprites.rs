use std::{collections::HashMap, path::Path};

use crate::{geometry::Rect, graphics::rendering::Renderer};

use super::rendering::TextureID;

#[derive(Debug)]
pub struct SpriteSystem {
    sprite_sheets: HashMap<SpriteSheetID, SpriteSheetData>,
    next_id: u32,
    scaling: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteSheetID(u32);

#[derive(Debug)]
pub enum LoadError {
    IoError(std::io::Error),
    JsonError(serde_json::error::Error),
}

#[derive(Debug, Clone)]
struct SpriteSheetData {
    texture_id: TextureID,
    sprites: Vec<Rect>,
    color_key: Option<(u8, u8, u8)>,
}

pub fn load_aseprite_sprite_sheet(
    json_path: &Path,
) -> Result<aseprite::SpritesheetData, LoadError> {
    let json_file = std::fs::File::open(json_path).map_err(|e| LoadError::IoError(e))?;
    serde_json::from_reader(json_file).map_err(|e| LoadError::JsonError(e))
}

pub fn aseprite_sprite_sheet_frames(sprite_sheet_data: &aseprite::SpritesheetData) -> Vec<Rect> {
    sprite_sheet_data
        .frames
        .iter()
        .map(|frame| Rect {
            x: frame.frame.x as i32,
            y: frame.frame.y as i32,
            w: frame.frame.w,
            h: frame.frame.h,
        })
        .collect()
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
        texture: TextureID,
        sprites: &[Rect],
        color_key: Option<(u8, u8, u8)>,
    ) -> SpriteSheetID {
        let id = self.make_id();
        let sprite_sheet = SpriteSheetData {
            texture_id: texture,
            sprites: Vec::from(sprites),
            color_key,
        };
        self.sprite_sheets.insert(id, sprite_sheet);
        id
    }

    /// Updates an existing sprite sheet in the system
    /// Used for hot reloading.
    pub fn reload_sprite_sheet(
        &mut self,
        id: SpriteSheetID,
        texture: TextureID,
        sprites: &[Rect],
        color_key: Option<(u8, u8, u8)>,
    ) {
        let sprite_sheet = SpriteSheetData {
            texture_id: texture,
            sprites: Vec::from(sprites),
            color_key,
        };
        *self.sprite_sheets.get_mut(&id).unwrap() = sprite_sheet;
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
            sprite_sheet.texture_id,
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
