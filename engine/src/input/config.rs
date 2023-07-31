use std::path::{Path, PathBuf};

use configparser::ini::Ini;

pub struct ProgramConfig {
    pub show_debug_ui: bool,
    pub monitor: u64,
    config: Ini,
    path: PathBuf,
}

impl ProgramConfig {
    pub fn from_file(path: &Path) -> Self {
        let mut config = Ini::new();
        if path.exists() {
            config.load(path).unwrap();
            ProgramConfig {
                show_debug_ui: config.getbool("Debug UI", "Show").unwrap().unwrap_or(false),
                monitor: config.getuint("Video", "Monitor").unwrap().unwrap_or(0),
                config,
                path: PathBuf::from(path),
            }
        } else {
            ProgramConfig {
                show_debug_ui: false,
                monitor: 0,
                config,
                path: PathBuf::from(path),
            }
        }
    }

    pub fn write_to_disk(&mut self) {
        self.config
            .set("Debug UI", "Show", Some(self.show_debug_ui.to_string()));
        self.config
            .set("Video", "Monitor", Some(self.monitor.to_string()));
        self.config.write(&self.path).unwrap();
    }
}
