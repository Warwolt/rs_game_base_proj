use chrono::Local;
use colored::*;
use log::{Level, Metadata, Record};

#[derive(Debug)]
pub struct LogStatement {
    pub level: Level,
    pub module: String,
    pub text: String,
    pub time: String,
}

struct CapturingLogger;

static mut CAPTURED_RECORDS: Vec<LogStatement> = Vec::new();

impl log::Log for CapturingLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if metadata.target().starts_with("hot_lib_reloader") {
            return false;
        }
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let time = Local::now().format("%H:%M:%S%.3f").to_string();
            let module = record.module_path().unwrap_or("");
            let text = record.args().to_string();
            unsafe {
                CAPTURED_RECORDS.push(LogStatement {
                    level: record.level(),
                    module: module.to_owned(),
                    text: text.clone(),
                    time: time.clone(),
                });
            }
            let level_string = record.level().to_string();
            let colored_level_string = match record.level() {
                Level::Error => level_string.red(),
                Level::Warn => level_string.yellow(),
                Level::Info => level_string.green(),
                Level::Debug => level_string.purple(),
                Level::Trace => level_string.cyan(),
            };
            println!("{} {} [{}] {}", time, colored_level_string, module, text);
        }
    }

    fn flush(&self) {}
}

use log::LevelFilter;

static LOGGER: CapturingLogger = CapturingLogger;

pub fn init_logging() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Trace))
        .unwrap();
}

pub(crate) fn get_log() -> &'static Vec<LogStatement> {
    unsafe { &CAPTURED_RECORDS }
}
