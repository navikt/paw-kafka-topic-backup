mod app_state;
mod config_utils;
mod database;
mod errors;
mod kafka;
mod logging;
mod metrics;
mod nais_http_apis;

use crate::app_state::AppState;
use crate::database::init_pg_pool::init_db;
use crate::kafka::config::ApplicationKafkaConfig;
use crate::kafka::hwm::HwmRebalanceHandler;
use crate::kafka::kafka_connection::create_kafka_consumer;
use crate::kafka::message_processor::KafkaMessage;
use crate::kafka::message_processor::prosesser_melding;
use crate::logging::init_log;
use crate::nais_http_apis::register_nais_http_apis;
use log::error;
use log::info;
use rdkafka::consumer::StreamConsumer;
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use tokio::signal::unix::{SignalKind, signal};

#[tokio::main]
async fn main() {
    // Set up panic handler to log panics before they crash the process
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("PANIC occurred: {}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!(
                "PANIC location: {}:{}:{}",
                location.file(),
                location.line(),
                location.column()
            );
        }
        if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("PANIC message: {}", s);
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("PANIC message: {}", s);
        }
    }));

    info!("Starter applikasjon");
    let _ = match run_app().await {
        Ok(_) => {
            info!("Applikasjonen avsluttet uten feil");
        }
        Err(e) => {
            error!("Feil ved kjøring av applikasjon, avslutter: {}", e);
            error!("Error details: {:?}", e);
            error!("Error source chain:");
            let mut source = e.source();
            let mut level = 1;
            while let Some(err) = source {
                error!("  Level {}: {}", level, err);
                source = err.source();
                level += 1;
            }
        }
    };
    info!("Main funksjon ferdig, applikasjon avsluttet");
}

async fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    init_log();

    // Initialize Prometheus metrics
    crate::metrics::init_metrics();
    info!("Prometheus metrics initialized");

    let app_state = Arc::new(AppState::new());
    let http_server_task = register_nais_http_apis(app_state.clone());
    info!("HTTP server startet");
    let pg_pool = init_db().await?;
    let stream = create_kafka_consumer(
        app_state.clone(),
        pg_pool.clone(),
        ApplicationKafkaConfig::new("hedelselogg_backup2_v1", "ssl"),
        &[
            "paw.arbeidssoker-hendelseslogg-v1",
            "paw.arbeidssoker-bekreftelse-v1",
            "paw.arbeidssoker-bekreftelse-paavegneav-v2",
        ],
    )?;
    let reader = read_all(pg_pool.clone(), stream);
    let signal = await_signal();
    app_state.set_has_started(true);
    info!("Alle tjenester startet, applikasjon kjører");
    tokio::select! {
        result = http_server_task => {
            match result {
                Ok(Ok(())) => info!("HTTP server stoppet."),
                Ok(Err(e)) => return Err(e),
                Err(join_error) => return Err(Box::new(join_error)),
            }
        }
        result = reader => {
            match result {
                Ok(()) => info!("Lesing av kafka topics stoppet."),
                Err(e) => return Err(e),
            }
        }
        result = signal => {
            match result {
                Ok(signal) => info!("Signal '{}' mottatt, avslutter....", signal),
                Err(e) => return Err(e.into()),
            }
        }
    }
    app_state.set_is_alive(false);
    let _ = pg_pool.close().await;
    info!("Pg pool lukket");
    Ok(())
}

async fn read_all(
    pg_pool: PgPool,
    stream: StreamConsumer<HwmRebalanceHandler>,
) -> Result<(), Box<dyn Error>> {
    loop {
        let msg = stream.recv().await?;
        let msg = KafkaMessage::from_borrowed_message(msg)?;
        prosesser_melding(pg_pool.clone(), msg).await?;
    }
}

async fn await_signal() -> Result<String, Box<dyn Error>> {
    let mut term_signal = signal(SignalKind::terminate())?;
    let mut interrupt_signal = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = term_signal.recv() => Ok("SIGTERM".to_string()),
        _ = interrupt_signal.recv() => Ok("SIGINT".to_string())
    }
}
