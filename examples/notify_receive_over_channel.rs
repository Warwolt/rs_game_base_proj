use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use notify::{Event, RecursiveMode, Result, Watcher};

fn main() -> Result<()> {
    // Setup communcation channels
    let (tx, rx): (Sender<Event>, Receiver<Event>) = mpsc::channel();

    // Automatically select the best implementation for your platform.
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            println!("notify callback, event: {:?}", event);
            tx.send(event).unwrap();
        }
        Err(e) => println!("watch error: {:?}", e),
    })?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    println!("ctrl+c to exit");
    loop {
        if let Ok(event) = rx.try_recv() {
            println!("main thread, event: {:?}", event);
        }
    }
}
