#[derive(Clone, Copy, Debug)]
pub struct AppState {
    pub is_alive: bool,
    pub is_ready: bool,
    pub has_started: bool,
}
