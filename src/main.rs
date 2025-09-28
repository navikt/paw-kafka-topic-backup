mod app_state;
mod config_utils;
mod database;
mod errors;
mod kafka;
mod logging;
mod nais_http_apis;

use crate::app_state::AppState;
use crate::database::create_tables;
use crate::database::init_pg_pool::init_db;
use crate::database::insert_data;
use crate::kafka::config::ApplicationKafkaConfig;
use crate::kafka::headers::extract_headers_as_json;
use crate::kafka::hwm::HwmRebalanceHandler;
use crate::kafka::kafka_connection::create_kafka_consumer;
use crate::logging::init_log;
use crate::nais_http_apis::register_nais_http_apis;
use log::error;
use log::info;
use rdkafka::Message;
use rdkafka::consumer::StreamConsumer;
use rdkafka::message::Headers;
use sqlx::PgPool;
use std::error::Error;
use std::process::exit;
use tokio::signal::unix::{SignalKind, signal};

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
    let http_server_task = register_nais_http_apis(app_state);
    info!("HTTP server startet");
    let pg_pool = init_db().await?;
    //let pg_pool = Arc::new(pg_pool);
    let _ = create_tables(&pg_pool).await?;
    let stream = create_kafka_consumer(
        pg_pool.clone(),
        ApplicationKafkaConfig::new("hedelselogg_backup2_v1", "ssl"),
        &["paw.arbeidssoker-hendelseslogg-v1"],
    )?;
    let reader = read_all(pg_pool.clone(), stream);
    let signal = await_signal();
    tokio::select! {
        result = http_server_task => {
            match result {
                Ok(Ok(())) => info!("HTTP server completed"),
                Ok(Err(e)) => error!("HTTP server failed: {}", e),
                Err(join_error) => error!("HTTP server task panicked: {}", join_error),
            }
        }
        result = reader => {
            match result {
                Ok(()) => info!("Reader completed"),
                Err(e) => error!("Reader failed: {}", e),
            }
        }
        result = signal => {
            match result {
                Ok(()) => info!("Signal received, shutting down..."),
                Err(e) => error!("Signal handler failed: {}", e),
            }
        }
    }
    let _ = pg_pool.close().await;
    info!("Pg pool lukket");
    Ok(())
}

async fn read_all(
    pg_pool: PgPool,
    stream: StreamConsumer<HwmRebalanceHandler>,
) -> Result<(), Box<dyn Error>> {
    let mut counter = 0;
    loop {
        match stream.recv().await {
            Err(e) => {
                error!("Kafka error: {}", e);
                exit(2);
            }
            Ok(msg) => {
                let mut tx = pg_pool.begin().await?;
                let _ = insert_data::insert_data(
                    &mut tx,
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp().to_millis().unwrap_or(0),
                    extract_headers_as_json(&msg)?,
                    msg.key().unwrap_or(&[]),
                    msg.payload().unwrap_or(&[]),
                )
                .await?;
                counter += 1;
                if counter % 1000 == 0 {
                    info!("Antall meldinger mottatt: {}", counter);
                    info!(
                        "Leste melding: topic={}, partition={}, offset={}, header={}, value_size={}",
                        msg.topic(),
                        msg.partition(),
                        msg.offset(),
                        msg.headers().map(|h| h.count()).get_or_insert(0),
                        msg.payload_len()
                    );
                }
            }
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
