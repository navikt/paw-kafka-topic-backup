mod logging;
mod app_state;
mod nais_http_apis;
mod postgres;
mod errors;
mod database_config;
mod table;

use std::error::Error;
use std::process::exit;
use crate::app_state::AppState;
use crate::nais_http_apis::register_nais_http_apis;
use log::info;
use log::error;
use sqlx::PgPool;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task;
use crate::logging::init_log;
use crate::table::create_table;

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
    let pg_pool = match init_db().await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Feil ved initiering av database: {}", e);
            exit(1);
        }
    };
    let _ = match create_table(&pg_pool).await {
        Ok(_) => {
            info!("Tabell opprettet eller eksisterer allerede");
            ()
        }
        Err(e) => {
            error!("Feil ved oppretting av tabell: {}", e);
            exit(1);
        }
    };
    let _ = match await_signal().await {
        Ok(_) => { () }
        Err(e) => {
            error!("Feil ved venting på signal: {}", e);
            ()
        }
    };
    pg_pool.close().await;
    info!("Pg pool lukket");
    Ok(())
}

async fn await_signal() -> Result<(), Box<dyn Error>> {
    let mut term_signal = signal(SignalKind::terminate())?;
    let mut interrupt_signal = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = term_signal.recv() => {
            info!("\nSIGTERM mottatt, avslutter...");
            Ok(())
        },
        _ = interrupt_signal.recv() => {
            info!("\nSIGINT mottatt, avslutter...");
            Ok(())
        },
    }
}

async fn init_db() -> Result<PgPool, Box<dyn Error>> {
    let db_config = database_config::get_database_config()?;
    info!("Database config: {:?}", db_config);
    let pg_pool = postgres::get_pg_pool(&db_config)?;
    info!("Postgres pool opprettet");
    Ok(pg_pool)
}
