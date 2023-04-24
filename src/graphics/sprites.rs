use std::collections::HashMap;

use crate::{
    geometry::Rect,
    graphics::rendering::{Renderer, TextureData},
};

#[derive(Debug)]
pub struct SpriteSystem {
    sprite_sheets: HashMap<SpriteSheetID, SpriteSheetData>,
    next_id: u32,
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
        }
    }

    pub fn add_spritesheet(
        &mut self,
        texture: TextureData,
        sprites: &[Rect],
        color_key: Option<(u8, u8, u8)>,
    ) -> SpriteSheetID {
        let id = self.make_id();
        let sprite_sheet = SpriteSheetData {
            texture,
            sprites: Vec::from(sprites),
            color_key,
        };
        self.sprite_sheets.insert(id, sprite_sheet);
        id
    }

    pub fn spritesheet_dimensions(&self, sprite_sheet: SpriteSheetID) -> (u32, u32) {
        let sprite_sheet = &self.sprite_sheets[&sprite_sheet];
        (sprite_sheet.texture.width, sprite_sheet.texture.height)
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
                w: sprite_rect.w,
                h: sprite_rect.h,
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
