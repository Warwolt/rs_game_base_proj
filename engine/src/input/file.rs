use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use itertools::Itertools;
use notify::{RecursiveMode, Watcher};

pub fn is_same_file(lhs: &Path, rhs: &Path) -> bool {
    same_file::is_same_file(lhs, rhs).unwrap_or(false)
}

pub struct FileWatcher {
    _file_watcher: notify::RecommendedWatcher,
    event_receiver: Receiver<notify::Event>,
    debounce_time: Duration,
    changed_files: Vec<PathBuf>,
    elapsed_time_ms: u128,
}

impl FileWatcher {
    /// Creates a file watcher for the `path` file or directory, that will
    /// filter out any repeated file changes events in `debounce_time` after the
    /// first file change received.
    pub fn new(path: &Path, debounce_time: Duration) -> Self {
        let (tx, rx): (Sender<notify::Event>, Receiver<notify::Event>) = mpsc::channel();
        let on_file_changed = move |result: notify::Result<notify::Event>| match result {
            Ok(event) => {
                tx.send(event).unwrap();
            }
            Err(e) => log::error!("file watch error: {:?}", e),
        };

        let mut file_watcher = notify::recommended_watcher(on_file_changed).unwrap();
        file_watcher.watch(path, RecursiveMode::Recursive).unwrap();

        FileWatcher {
            _file_watcher: file_watcher,
            event_receiver: rx,
            debounce_time,
            changed_files: Vec::new(),
            elapsed_time_ms: 0,
        }
    }

    pub fn update(&mut self, delta_time_ms: u128) -> Vec<PathBuf> {
        // Add all new events to vector
        self.changed_files.append(
            &mut self
                .event_receiver
                .try_recv()
                .map(|event| event.paths)
                .unwrap_or(Vec::new()),
        );

        // Track debounce time while new events exist
        if !self.changed_files.is_empty() {
            self.elapsed_time_ms += delta_time_ms;
        }

        // Return events after debouncing
        if self.elapsed_time_ms >= self.debounce_time.as_millis() {
            self.elapsed_time_ms = 0;
            self.changed_files.drain(0..).unique().collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_file_watcher(debounce_time: Duration, rx: Receiver<notify::Event>) -> FileWatcher {
        let on_file_changed = |_: notify::Result<notify::Event>| {};
        let file_watcher = notify::recommended_watcher(on_file_changed).unwrap();
        FileWatcher {
            _file_watcher: file_watcher,
            event_receiver: rx,
            debounce_time,
            changed_files: Vec::new(),
            elapsed_time_ms: 0,
        }
    }

    fn send_file_update(tx: &Sender<notify::Event>, path: &str) {
        tx.send(notify::Event {
            kind: notify::EventKind::Modify(notify::event::ModifyKind::Any),
            paths: vec![PathBuf::from(path)],
            attrs: notify::event::EventAttributes::new(),
        })
        .unwrap();
    }

    #[test]
    fn initially_contains_no_changed_files() {
        let (_, rx): (Sender<notify::Event>, Receiver<notify::Event>) = mpsc::channel();
        let mut file_watcher = new_test_file_watcher(Duration::from_millis(100), rx);

        let updated_files = file_watcher.update(0);

        assert!(updated_files.is_empty());
    }

    #[test]
    fn does_not_return_changed_files_if_debounce_period_has_not_elapsed() {
        let (tx, rx): (Sender<notify::Event>, Receiver<notify::Event>) = mpsc::channel();
        let mut file_watcher = new_test_file_watcher(Duration::from_millis(100), rx);

        send_file_update(&tx, "./resources/my_image.png");
        let updated_files = file_watcher.update(0);

        assert!(updated_files.is_empty());
    }

    #[test]
    fn returns_changed_files_after_debounce_period_has_elapsed() {
        let (tx, rx): (Sender<notify::Event>, Receiver<notify::Event>) = mpsc::channel();
        let mut file_watcher = new_test_file_watcher(Duration::from_millis(100), rx);

        send_file_update(&tx, "./resources/my_image.png");
        send_file_update(&tx, "./resources/my_image2.png");
        file_watcher.update(0);
        let updated_files = file_watcher.update(100);

        assert_eq!(
            updated_files,
            vec![
                PathBuf::from("./resources/my_image.png"),
                PathBuf::from("./resources/my_image2.png")
            ]
        );
    }

    #[test]
    fn repeated_file_changes_are_filtered_out() {
        let (tx, rx): (Sender<notify::Event>, Receiver<notify::Event>) = mpsc::channel();
        let mut file_watcher = new_test_file_watcher(Duration::from_millis(100), rx);

        send_file_update(&tx, "./resources/my_image.png");
        send_file_update(&tx, "./resources/my_image.png");
        send_file_update(&tx, "./resources/my_image.png");
        file_watcher.update(0);
        let updated_files = file_watcher.update(100);

        assert_eq!(
            updated_files,
            vec![PathBuf::from("./resources/my_image.png"),]
        );
    }
}
