use std::sync::Arc;

use axum::{Router, extract::{State, WebSocketUpgrade}, response::IntoResponse, routing::get};
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
    
}