use std::{collections::HashMap, path::Path};

use sdl2::mixer::{Chunk, Music};

pub struct AudioPlayer<'a> {
    sounds: HashMap<SoundID, Chunk>,
    tracks: HashMap<MusicID, Music<'a>>,
    next_sound_id: u32,
    next_music_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SoundID(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MusicID(u32);

impl<'a> AudioPlayer<'a> {
    pub fn new() -> Self {
        AudioPlayer {
            sounds: HashMap::new(),
            tracks: HashMap::new(),
            next_sound_id: 0,
            next_music_id: 0,
        }
    }

    pub fn add_sound(&mut self, path: &Path) -> SoundID {
        let id = self.generate_sound_id();
        let chunk = sdl2::mixer::Chunk::from_file(path).unwrap();
        self.sounds.insert(id, chunk);
        id
    }

    pub fn add_music(&mut self, path: &Path) -> MusicID {
        let id = self.generate_music_id();
        let music = sdl2::mixer::Music::from_file(path).unwrap();
        self.tracks.insert(id, music);
        id
    }

    pub fn play_sound(&self, sound: SoundID) {
        let chunk = &self.sounds[&sound];
        sdl2::mixer::Channel::all().play(chunk, 0).unwrap();
    }

    pub fn play_music(&self, music: MusicID) {
        let music = &self.tracks[&music];
        music.play(-1).unwrap();
    }

    pub fn pause_music(&self) {
        sdl2::mixer::Music::pause();
    }

    pub fn resume_music(&self) {
        sdl2::mixer::Music::resume();
    }

    pub fn music_is_paused(&self) -> bool {
        sdl2::mixer::Music::is_paused()
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
