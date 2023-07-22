pub mod audio_reload;
pub mod sprite_reload;

use std::{path::PathBuf, time::Duration};

use crate::{
    audio::AudioSystem,
    graphics::{animation::AnimationSystem, rendering::Renderer, sprites::SpriteSystem},
    input::file::FileWatcher,
};

use self::{audio_reload::AudioReloader, sprite_reload::AsepriteReloader};

pub struct ResourceReloader {
    file_watcher: FileWatcher,
    audio_reloader: AudioReloader,
    sprite_reloader: AsepriteReloader,
}

impl ResourceReloader {
    pub fn new(resource_dir: &PathBuf) -> Self {
        let file_watcher_debounce = Duration::from_millis(1000);
        ResourceReloader {
            file_watcher: FileWatcher::new(&resource_dir, file_watcher_debounce),
            audio_reloader: AudioReloader::new(),
            sprite_reloader: AsepriteReloader::new(),
        }
    }

    pub fn update(
        &mut self,
        delta_time_ms: u128,
        renderer: &mut Renderer,
        sprite_system: &mut SpriteSystem,
        animation_system: &mut AnimationSystem,
        audio_player: &mut AudioSystem,
    ) {
        let updated_files = self.file_watcher.update(delta_time_ms);
        self.audio_reloader.update(&updated_files, audio_player);
        self.sprite_reloader
            .update(&updated_files, renderer, sprite_system, animation_system);
    }

    pub fn audio_reloader(&mut self) -> &mut AudioReloader {
        &mut self.audio_reloader
    }

    pub fn sprite_reloader(&mut self) -> &mut AsepriteReloader {
        &mut self.sprite_reloader
    }
}
