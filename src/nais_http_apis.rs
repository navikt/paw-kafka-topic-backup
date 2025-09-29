use std::sync::Arc;

use crate::app_state::AppState;
use axum::extract::State;
use axum::{Router, http::StatusCode, routing::get};
use prometheus::{Encoder, TextEncoder};
use tokio::task::JoinHandle;

pub fn register_nais_http_apis(
    app_state: Arc<AppState>,
) -> JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {
    tokio::spawn(async move {
        let routes = routes(app_state);
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
        axum::serve(listener, routes).await?;
        Ok(())
    })
}

fn routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/internal/isAlive", get(is_alive))
        .route("/internal/isReady", get(is_ready))
        .route("/internal/hasStarted", get(has_started))
        .route("/internal/metrics", get(prometheus))
        .with_state(app_state)
}

async fn is_alive(State(app_state): State<Arc<AppState>>) -> (StatusCode, &'static str) {
    if app_state.get_is_alive() {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable")
    }
}

async fn is_ready(State(app_state): State<Arc<AppState>>) -> (StatusCode, &'static str) {
    if app_state.get_is_ready() {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable")
    }
}

async fn has_started(State(app_state): State<Arc<AppState>>) -> (StatusCode, &'static str) {
    if app_state.get_has_started() {
        (StatusCode::OK, "ok")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable")
    }
}

async fn prometheus() -> (StatusCode, [(&'static str, &'static str); 1], String) {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metrics = prometheus::gather();
    encoder.encode(&metrics, &mut buffer).unwrap();
    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        String::from_utf8(buffer).unwrap(),
    )
}
