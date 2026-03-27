use std::sync::Arc;

use axum::{Router, extract::{State, WebSocketUpgrade, ws::{Message, WebSocket}}, response::IntoResponse, routing::get};
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;
use tokio::sync::broadcast;





struct AppState {
    tx: broadcast::Sender<String>
}



#[tokio::main]
async fn main() {

    // 1. if a client is too slow and lags 16 messages it will drop
    let (tx, rx) = broadcast::channel(16);
    let app_state = Arc::new(AppState { tx});

    // 2. the router
    let app = Router::new()
        .route("/chat", get(ws_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server started ae ws://127.0.0.1:3000/chat");
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state) : State<Arc<AppState>>
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    // spawn a green thread for each ws connection
    tokio::spawn(async move {
        loop {
            tokio::select! {
                // BRANCH A: Listen for messages from the Hoppscotch Client
                // This is the "Stream" half
                Some(Ok(msg)) = receiver.next() => {
                    if let Message::Text(text) = msg {
                        // When we get a message, broadcast it to EVERYONE
                        let _ = state.tx.send(format!("Anonymous: {}", text));
                    }
                }

                // BRANCH B: Listen for messages from the Global Megaphone
                // This is the "Broadcast" half
                Ok(msg) = rx.recv() => {
                    // Send the global message out to this specific client
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        // If sending fails (client disconnected), exit the loop
                        break;
                    }
                }
            }
        }
    });
}