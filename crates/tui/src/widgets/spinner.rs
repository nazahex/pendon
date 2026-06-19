use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use crossterm::{
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};

use crate::assets::SPINNER_FRAMES_ASCII;
use crate::theme::Theme;

pub struct Spinner {
    stop: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Spinner {
    pub fn start(message: String, _theme: Theme) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let handle = thread::spawn(move || {
            let mut i = 0usize;
            let frames = SPINNER_FRAMES_ASCII;
            while !stop2.load(Ordering::Relaxed) {
                let mut err = std::io::stderr();
                let frame = frames[i % frames.len()];
                let line = format!("{} {}", frame, message);
                let _ = execute!(err, Clear(ClearType::CurrentLine), Print(line), Print("\r"));
                i = i.wrapping_add(1);
                thread::sleep(Duration::from_millis(80));
            }
            let mut err = std::io::stderr();
            let _ = execute!(err, Clear(ClearType::CurrentLine));
        });
        Spinner {
            stop,
            handle: Some(handle),
        }
    }

    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}
