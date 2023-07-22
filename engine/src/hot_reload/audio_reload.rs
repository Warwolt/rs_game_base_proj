use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    audio::{AudioSystem, SoundID},
    input::file::is_same_file,
};

pub struct AudioReloader {
    sounds: HashMap<SoundID, PathBuf>,
}

impl AudioReloader {
    pub fn new() -> Self {
        AudioReloader {
            sounds: HashMap::new(),
        }
    }

    pub fn register_sound(&mut self, id: SoundID, path: &Path) {
        assert!(
            !self.sounds.contains_key(&id),
            "sound with ID {:?} already registered",
            id
        );
        self.sounds.insert(id, PathBuf::from(path));
    }

    pub fn update(&self, updated_files: &[PathBuf], audio_player: &mut AudioSystem) {
        for updated_file in updated_files {
            // if reloader containts file, reload sound
            for (id, path) in &self.sounds {
                if is_same_file(&updated_file, path) {
                    audio_player.reload_sound(*id, path);
                    continue;
                }
            }
        }
    }
}
