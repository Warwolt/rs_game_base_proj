use std::{collections::HashMap, path::Path};

use sdl2::mixer::{Chunk, Music};

pub struct AudioSystem<'a> {
    sounds: HashMap<SoundID, Chunk>,
    tracks: HashMap<MusicID, Music<'a>>,
    next_sound_id: u32,
    next_music_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundID(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MusicID(u32);

impl<'a> AudioSystem<'a> {
    pub fn new() -> Self {
        AudioSystem {
            sounds: HashMap::new(),
            tracks: HashMap::new(),
            next_sound_id: 0,
            next_music_id: 0,
        }
    }

    /// Add a new sound to the system so that it can be played
    #[allow(dead_code)]
    pub fn add_sound(&mut self, path: &Path) -> SoundID {
        let id = self.generate_sound_id();
        let mut chunk = sdl2::mixer::Chunk::from_file(path).unwrap();
        chunk.set_volume(128 / 2);
        self.sounds.insert(id, chunk);
        id
    }

    /// Load a new sound to an existing ID, used for hot reloading.
    #[allow(dead_code)]
    pub fn reload_sound(&mut self, id: SoundID, path: &Path) {
        log::info!("Reloading sound from \"{}\"", path.display());
        if let Some(sound) = self.sounds.get_mut(&id) {
            *sound = sdl2::mixer::Chunk::from_file(path).unwrap();
        } else {
            panic!("Trying to reload sound with non-registered id {}", id.0);
        }
    }

    #[allow(dead_code)]
    pub fn add_music(&mut self, path: &Path) -> MusicID {
        let id = self.generate_music_id();
        let music = sdl2::mixer::Music::from_file(path).unwrap();
        self.tracks.insert(id, music);
        id
    }

    #[allow(dead_code)]
    pub fn play_sound(&self, sound: SoundID) {
        let chunk = &self.sounds[&sound];
        sdl2::mixer::Channel::all().play(chunk, 0).unwrap();
    }

    #[allow(dead_code)]
    pub fn play_music(&self, music: MusicID) {
        let music = &self.tracks[&music];
        music.play(-1).unwrap();
    }

    #[allow(dead_code)]
    pub fn pause_music(&self) {
        sdl2::mixer::Music::pause();
    }

    #[allow(dead_code)]
    pub fn resume_music(&self) {
        sdl2::mixer::Music::resume();
    }

    #[allow(dead_code)]
    pub fn music_is_paused(&self) -> bool {
        sdl2::mixer::Music::is_paused()
    }

    #[allow(dead_code)]
    pub fn music_is_playing(&self) -> bool {
        sdl2::mixer::Music::is_playing()
    }

    fn generate_sound_id(&mut self) -> SoundID {
        let id = self.next_sound_id;
        self.next_sound_id += 1;
        SoundID(id)
    }

    fn generate_music_id(&mut self) -> MusicID {
        let id = self.next_music_id;
        self.next_music_id += 1;
        MusicID(id)
    }
}
