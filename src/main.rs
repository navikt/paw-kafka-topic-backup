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
use rdkafka::Message;
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use tokio::signal::unix::{SignalKind, signal};
use opentelemetry::trace::{TraceContextExt, Tracer, FutureExt};
use opentelemetry::{global, Context, KeyValue};

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
    
    // Initialize OpenTelemetry tracing
    init_tracing().await?;
    info!("OpenTelemetry tracing initialized");

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
        &["paw.arbeidssoker-hendelseslogg-v1"],
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
    let tracer = global::tracer("kafka-consumer");
    
    loop {
        let borrowed_msg = stream.recv().await?;
        
        // Extract trace context from Kafka headers
        let parent_context = extract_trace_context(&borrowed_msg);
        
        // Collect message info before moving borrowed_msg
        let topic = borrowed_msg.topic().to_string();
        let partition = borrowed_msg.partition();
        let offset = borrowed_msg.offset();
        
        // Create span for message processing
        let span = tracer
            .span_builder(format!("{} process", topic))
            .with_attributes(vec![
                KeyValue::new("messaging.system", "kafka"),
                KeyValue::new("messaging.operation", "process"),
                KeyValue::new("messaging.destination.name", topic),
                KeyValue::new("messaging.destination.partition.id", partition.to_string()),
                KeyValue::new("messaging.kafka.message.offset", offset),
            ])
            .start_with_context(&tracer, &parent_context);
        
        // Create context with the span
        let span_context = Context::current_with_span(span);
        
        // Process message within the span context
        let result: Result<(), Box<dyn Error>> = async {
            let msg = KafkaMessage::from_borrowed_message(borrowed_msg)?;
            prosesser_melding(pg_pool.clone(), msg).await
        }
        .with_context(span_context.clone())
        .await;
        
        // Update span status based on result
        let span = span_context.span();
        match &result {
            Ok(_) => {
                span.set_status(opentelemetry::trace::Status::Ok);
            }
            Err(e) => {
                span.set_status(opentelemetry::trace::Status::error(format!("Message processing failed: {}", e)));
                span.record_error(e.as_ref());
            }
        }
        
        result?;
    }
}

async fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry_otlp::WithExportConfig;
    use opentelemetry_sdk::{Resource, trace as sdktrace};
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    use opentelemetry::trace::TracerProvider;
    
    // Get OTLP endpoint from environment or use default
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());
    
    // Initialize OpenTelemetry tracer
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&otlp_endpoint)
        )
        .with_trace_config(
            sdktrace::Config::default().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "paw-kafka-topic-backup"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ]))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    
    // Set global tracer provider
    let _ = global::set_tracer_provider(tracer_provider.clone());
    
    // Get the tracer from the provider
    let tracer = tracer_provider.tracer("paw-kafka-topic-backup");
    
    // Set up tracing subscriber with OpenTelemetry layer
    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()?;
    
    Ok(())
}

fn extract_trace_context(msg: &rdkafka::message::BorrowedMessage) -> Context {
    use opentelemetry::propagation::{Extractor, TextMapPropagator};
    use opentelemetry_sdk::propagation::TraceContextPropagator;
    use rdkafka::message::Headers;
    
    struct KafkaHeaderExtractor<'a>(&'a rdkafka::message::BorrowedMessage<'a>);
    
    impl<'a> Extractor for KafkaHeaderExtractor<'a> {
        fn get(&self, key: &str) -> Option<&str> {
            if let Some(headers) = self.0.headers() {
                for i in 0..headers.count() {
                    let header = headers.get(i);
                    if header.key == key {
                        if let Some(value) = header.value {
                            return std::str::from_utf8(value).ok();
                        }
                    }
                }
            }
            None
        }
        
        fn keys(&self) -> Vec<&str> {
            if let Some(headers) = self.0.headers() {
                let mut keys = Vec::new();
                for i in 0..headers.count() {
                    let header = headers.get(i);
                    keys.push(header.key);
                }
                keys
            } else {
                Vec::new()
            }
        }
    }
    
    let propagator = TraceContextPropagator::new();
    let extractor = KafkaHeaderExtractor(msg);
    propagator.extract(&extractor)
}

async fn await_signal() -> Result<String, Box<dyn Error>> {
    let mut term_signal = signal(SignalKind::terminate())?;
    let mut interrupt_signal = signal(SignalKind::interrupt())?;
    tokio::select! {
        _ = term_signal.recv() => Ok("SIGTERM".to_string()),
        _ = interrupt_signal.recv() => Ok("SIGINT".to_string())
    }
}
