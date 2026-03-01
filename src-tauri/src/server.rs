use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::events::{is_valid_event, ClippyEvent};
use crate::sounds;
use crate::tray::TrayState;

/// Shared state for axum handlers
#[derive(Clone)]
pub struct AppState {
    pub app_handle: Arc<AppHandle>,
}

/// Start the HTTP server on 127.0.0.1:9999
pub async fn start_server(app_handle: AppHandle) {
    let state = AppState {
        app_handle: Arc::new(app_handle),
    };

    let app = Router::new()
        .route("/complete", get(handle_complete))
        .route("/error", get(handle_error))
        .route("/attention", get(handle_attention))
        .route("/stop", get(handle_stop))
        .route("/session-end", get(handle_session_end))
        .route("/health", get(handle_health))
        .with_state(state);

    let addr = "127.0.0.1:9999";
    log::info!("Clippy HTTP server starting on {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!(
                "Failed to bind to {}: {}. Is another instance running?",
                addr,
                e
            );
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        log::error!("HTTP server error: {}", e);
    }
}

/// Emit a clippy event to the webview
fn emit_event(state: &AppState, event_type: &str) {
    debug_assert!(is_valid_event(event_type), "Invalid event type: {}", event_type);
    let payload = ClippyEvent {
        event_type: event_type.to_string(),
    };
    log::info!("Event received: {}", event_type);

    // Check mute state before playing sound
    let tray_state = state.app_handle.state::<Arc<TrayState>>();
    if !tray_state.is_muted() {
        sounds::play_event_sound(event_type);
    } else {
        log::info!("Sound muted, skipping playback for: {}", event_type);
    }

    // Events override manual hide — ensure agent is visible
    tray_state.set_visible(true);
    let _ = state.app_handle.emit("clippy-visibility", true);

    // Always emit to webview (animation still plays even when muted)
    if let Err(e) = state.app_handle.emit("clippy-event", &payload) {
        log::warn!("Failed to emit clippy-event: {}", e);
    }
}

async fn handle_complete(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "complete");
    (StatusCode::OK, "OK")
}

async fn handle_error(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "error");
    (StatusCode::OK, "OK")
}

async fn handle_attention(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "attention");
    (StatusCode::OK, "OK")
}

async fn handle_stop(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "stop");
    (StatusCode::OK, "OK")
}

async fn handle_session_end(State(state): State<AppState>) -> impl IntoResponse {
    emit_event(&state, "session-end");
    (StatusCode::OK, "OK")
}

async fn handle_health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "app": "clippy-awakens",
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}
