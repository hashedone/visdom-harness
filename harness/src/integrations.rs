use std::time::Duration;

use axum::extract::WebSocketUpgrade;
use axum::extract::ws::{Message, WebSocket};
use axum::response::IntoResponse;
use axum::extract::State;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{debug, info, warn};

use crate::AppState;
use crate::llm::LlmClient;

/// Interval between server-sent pings to keep connections alive and detect dead clients.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Handler for `GET /integrations/connect`.
///
/// Upgrades the HTTP connection to a WebSocket, registers the sender in the
/// `IntegrationRegistry`, and spawns a task to drive the connection lifecycle.
pub async fn connect<L: LlmClient>(
    State(state): State<AppState<L>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket<L: LlmClient>(socket: WebSocket, state: AppState<L>) {
    let (tx, rx) = mpsc::unbounded_channel::<Message>();

    // Register this connection.
    {
        let mut guard = state
            .integrations
            .senders
            .lock()
            .expect("registry lock poisoned");
        guard.push(tx);
        info!(
            connections = guard.len(),
            "integration connected"
        );
    }

    drive_connection(socket, rx, state).await;
}

/// Drives a single WebSocket connection:
/// - Sends outbound messages queued via the channel.
/// - Reads inbound messages from the client (currently logged, not yet routed).
/// - Sends a WebSocket ping every `HEARTBEAT_INTERVAL`.
/// - Returns when the connection closes or an error occurs; the dead sender
///   will be pruned from the registry on the next `fan_out` call.
async fn drive_connection<L: LlmClient>(
    mut socket: WebSocket,
    mut rx: mpsc::UnboundedReceiver<Message>,
    state: AppState<L>,
) {
    let mut heartbeat = time::interval(HEARTBEAT_INTERVAL);
    // Skip the immediate first tick so the ping doesn't fire on connect.
    heartbeat.tick().await;

    loop {
        tokio::select! {
            // Outbound: send queued messages to the client.
            Some(msg) = rx.recv() => {
                if socket.send(msg).await.is_err() {
                    debug!("integration send failed — connection closed");
                    break;
                }
            }

            // Inbound: receive messages from the integration.
            result = socket.recv() => {
                match result {
                    Some(Ok(Message::Text(text))) => {
                        debug!(text = %text, "integration message received (routing not yet implemented)");
                        // TODO(S07+): route inbound messages to the loop dispatcher.
                        route_inbound(&state, text.to_string()).await;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        debug!("integration pong received");
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        info!("integration disconnected");
                        break;
                    }
                    Some(Ok(_)) => {
                        // Binary, Ping — ignore for now.
                    }
                    Some(Err(e)) => {
                        warn!(error = %e, "integration WebSocket error");
                        break;
                    }
                }
            }

            // Heartbeat: send a ping to keep the connection alive.
            _ = heartbeat.tick() => {
                if socket.send(Message::Ping(vec![].into())).await.is_err() {
                    debug!("integration heartbeat failed — connection closed");
                    break;
                }
                debug!("integration heartbeat sent");
            }
        }
    }
}

/// Placeholder for inbound message routing.
/// Will be wired to the loop dispatcher once questions/answers are implemented.
async fn route_inbound<L: LlmClient>(_state: &AppState<L>, text: String) {
    // Inbound message routing is not yet implemented.
    // When the question/answer model is built (S07+), this function will:
    // 1. Deserialize the message as an Answer (carrying question_id, project_id, text).
    // 2. Persist the answer to the DB.
    // 3. Re-trigger the reasoning loop for the relevant project.
    debug!(text = %text, "inbound message received — routing not yet implemented");
}
