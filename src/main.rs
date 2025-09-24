mod logging;
mod app_state;
mod nais_http_apis;
mod postgres;
mod errors;
mod database_config;
mod table;
mod kafka_connection;

use crate::app_state::AppState;
use crate::kafka_connection::create_kafka_consumer;
use crate::logging::init_log;
use crate::nais_http_apis::register_nais_http_apis;
use crate::table::create_table;
use log::error;
use log::info;
use rdkafka::consumer::{StreamConsumer};
use sqlx::PgPool;
use std::error::Error;
use std::process::exit;
use tokio::signal::unix::{signal, SignalKind};
use tokio::task;

#[tokio::main]
async fn main() {
    let _x = match run_app().await {
        Ok(_) => {}
        Err(e) => {
            error!("Feil ved kjøring av applikasjon, avslutter: {}", e);
            exit(1);
        }
    };
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    init_log();
    let app_state = AppState {
        is_alive: true,
        is_ready: true,
        has_started: true,
    };
    task::spawn(register_nais_http_apis(app_state));
    info!("HTTP server startet");
    let pg_pool = init_db().await?;
    let _ = create_table(&pg_pool).await?;
    let stream = create_kafka_consumer(
        "hendelselogg-backup-2-v1",
        &["paw.arbeidssoker-hendelseslogg-v1"],
        false
    )?;
    task::spawn(read_all(stream));
    let _ = await_signal().await?;
    pg_pool.close().await;
    info!("Pg pool lukket");
    Ok(())
}

async fn read_all(stream: StreamConsumer) {
    let mut counter = 0;
    loop {
        match stream.recv().await {
            Err(e) => {
                error!("Kafka error: {}", e);
                exit(2);
            },
            Ok(_) => {
                counter += 1;
            }
        }
        if counter % 1000 == 0 {
            info!("Antall meldinger mottatt: {}", counter);
        }
    }
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
    let pg_pool = postgres::get_pg_pool(&db_config).await?;
    info!("Postgres pool opprettet");
    Ok(pg_pool)
}
