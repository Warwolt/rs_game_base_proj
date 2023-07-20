use crate::graphics::{
    animation::{reload_aseperite_sprite_sheet_animation, AnimationID, AnimationSystem},
    rendering::{self, Renderer, TextureID},
    sprites::{
        aseprite_sprite_sheet_frames, load_aseprite_sprite_sheet, SpriteSheetID, SpriteSystem,
    },
};
use crate::input::file::is_same_file;
use ::aseprite::SpritesheetData;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct AsepriteReloader {
    watched_sprite_sheets: HashMap<PathBuf, AsepriteWatchData>,
}

#[derive(Debug)]
struct AsepriteWatchData {
    texture_path: PathBuf,
    texture_id: TextureID,
    sprite_sheet_id: SpriteSheetID,
    animations: Vec<AsepriteAnimationData>,
    should_update: bool,
}

#[derive(Debug)]
struct AsepriteAnimationData {
    animation_id: AnimationID,
    frame_tag_name: String,
}

impl AsepriteReloader {
    pub fn new() -> Self {
        AsepriteReloader {
            watched_sprite_sheets: HashMap::new(),
        }
    }

    pub fn register_aseprite_sprite_sheet(
        &mut self,
        texture_path: &Path,
        json_path: &Path,
        texture_id: TextureID,
        sprite_sheet_id: SpriteSheetID,
    ) {
        let previous = self.watched_sprite_sheets.insert(
            PathBuf::from(json_path),
            AsepriteWatchData {
                texture_path: PathBuf::from(texture_path),
                texture_id,
                sprite_sheet_id,
                animations: Vec::new(),
                should_update: false,
            },
        );
        if previous.is_some() {
            panic!(
                "Trying to register already existing sprite sheet \"{}\"",
                json_path.display()
            );
        }
    }

    pub fn register_aseprite_animation(
        &mut self,
        json_path: &Path,
        animation_id: AnimationID,
        frame_tag_name: &str,
    ) {
        if let Some(watched_data) = self.watched_sprite_sheets.get_mut(json_path) {
            watched_data.animations.push(AsepriteAnimationData {
                animation_id,
                frame_tag_name: String::from(frame_tag_name),
            });
        } else {
            panic!(
                "Trying to register animations for non-existing sprite sheet with path {}. Did you forget to call register_aseprite_sprite_sheet?",
                json_path.display()
            );
        }
    }

    pub fn update(
        &mut self,
        updated_files: &[PathBuf],
        renderer: &mut Renderer,
        sprite_system: &mut SpriteSystem,
        animation_system: &mut AnimationSystem,
    ) {
        for updated_file_path in updated_files {
            for (json_path, watched_sprite_sheet) in &mut self.watched_sprite_sheets {
                watched_sprite_sheet.should_update =
                    is_same_file(&updated_file_path, &watched_sprite_sheet.texture_path)
                        || is_same_file(&updated_file_path, &json_path);
            }
        }

        for (json_path, watched_sprite_sheet) in &self.watched_sprite_sheets {
            if watched_sprite_sheet.should_update {
                log::info!("Reloading sprite sheet from \"{}\"", json_path.display());
                if let Ok(sprite_sheet_data) = load_aseprite_sprite_sheet(&json_path) {
                    self.reload_sprite_sheet(
                        renderer,
                        sprite_system,
                        &sprite_sheet_data,
                        &watched_sprite_sheet,
                    );
                    for animation in &watched_sprite_sheet.animations {
                        reload_aseperite_sprite_sheet_animation(
                            animation.animation_id,
                            animation_system,
                            &sprite_sheet_data,
                            &animation.frame_tag_name,
                        );
                        animation_system.restart_animation(animation.animation_id);
                    }
                }
            }
        }

        for (_, watched_sprite_sheet) in &mut self.watched_sprite_sheets {
            watched_sprite_sheet.should_update = false;
        }
    }

    fn reload_sprite_sheet(
        &self,
        renderer: &mut Renderer,
        sprite_system: &mut SpriteSystem,
        sprite_sheet_data: &SpritesheetData,
        watched_sprite_sheet: &AsepriteWatchData,
    ) -> Option<()> {
        // Reload sprite sheet texture
        rendering::reload_texture_from_image_path(
            watched_sprite_sheet.texture_id,
            renderer,
            &watched_sprite_sheet.texture_path,
        )
        .ok()?;

        // Reload sprite sheet data
        let frames = aseprite_sprite_sheet_frames(&sprite_sheet_data);
        sprite_system.reload_sprite_sheet(
            watched_sprite_sheet.sprite_sheet_id,
            watched_sprite_sheet.texture_id,
            &frames,
            None,
        );

        Some(())
    }
}
