use std::io::Write;

pub fn init_logging() {
    simple_logger::SimpleLogger::new()
        .with_module_level("hot_lib_reloader", log::LevelFilter::Error)
        .init()
        .unwrap();
}
