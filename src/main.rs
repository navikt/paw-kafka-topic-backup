mod logging;
mod app_state;
mod nais_http_apis;
mod postgres;
mod errors;

use std::error::Error;
use std::process::exit;
use crate::app_state::AppState;
use crate::nais_http_apis::register_nais_http_apis;
use log::info;
use log::error;
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
    let db_config = postgres::get_database_config()?;
    info!("Database config: {:?}", db_config);
    task::spawn(register_nais_http_apis(app_state));
    info!("HTTP server startet");
    let pg_pool = postgres::get_pg_pool(&db_config)?;
    info!("Postgres pool opprettet");
    let mut term_signal = signal(SignalKind::terminate())?;
    let mut interrupt_signal = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = term_signal.recv() => {info!("\nSIGTERM mottatt, avslutter...");},
        _ = interrupt_signal.recv() => {info!("\nSIGINT mottatt, avslutter...");},
    }
    Ok(())
}
