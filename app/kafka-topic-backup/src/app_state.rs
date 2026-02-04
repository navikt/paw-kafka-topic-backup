use std::sync::atomic::AtomicBool;

#[derive(Debug)]
pub struct AppState {
    pub is_alive: AtomicBool,
    pub is_ready: AtomicBool,
    pub has_started: AtomicBool,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            is_alive: AtomicBool::new(true),
            is_ready: AtomicBool::new(true),
            has_started: AtomicBool::new(true),
        }
    }

    pub fn set_is_alive(&self, value: bool) {
        self.is_alive
            .store(value, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn set_has_started(&self, value: bool) {
        self.has_started
            .store(value, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_is_alive(&self) -> bool {
        self.is_alive.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_is_ready(&self) -> bool {
        self.is_ready.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_has_started(&self) -> bool {
        self.has_started.load(std::sync::atomic::Ordering::Relaxed)
    }
}
