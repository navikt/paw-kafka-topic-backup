mod logging;
mod app_state;
mod nais_http_apis;

use crate::app_state::AppState;
use crate::nais_http_apis::register_nais_http_apis;
use log::info;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task;
use crate::logging::init_log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_log();
    let app_state = AppState {
        is_alive: true,
        is_ready: true,
        has_started: true,
    };
    task::spawn(register_nais_http_apis(app_state));
    info!("HTTP server startet");

    let mut term_signal = signal(SignalKind::terminate())?;
    let mut interrupt_signal = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = term_signal.recv() => {println!("\nSIGTERM mottatt, avslutter...");},
        _ = interrupt_signal.recv() => {println!("\nSIGINT mottatt, avslutter...");},
    }
    Ok(())
}
