#![feature(try_blocks)]

use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};

use crate::{app::App, namespace::messages::export_types};

pub mod app;
pub mod namespace;
pub mod store;
pub mod ws;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    export_types("./client/messages.ts");

    let app = App::new();

    app.new_namespace(
        String::from("nathan"),
        std::env::var("WRITE_KEY").unwrap_or("soup".into()),
    )
    .await;
    app.new_namespace(
        String::from("sandbox"),
        String::from(""), // no write key
    )
    .await;
    app.new_namespace(
        String::from("tictactoe"),
        String::from("x"), // no write key
    )
    .await;

    let router = Router::new()
        .route("/", get(root))
        .route("/read/:ns/:store", get(read_store))
        .route("/write/:ns/:wk/:store", post(write_store))
        .route("/ws/:ns", get(handle_ws_read))
        .route("/ws/:ns/:wp", get(handle_ws_write))
        .with_state(app);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port");

    println!("Listening on 3000");

    axum::serve(listener, router).await.expect("server failed");
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn read_store(State(app): State<App>, Path((ns, store)): Path<(String, String)>) -> Response {
    match app.read_store(&ns, &store).await {
        Some(data) => data.into_response(),
        None => "Store not found".into_response(),
    }
}

async fn write_store(
    State(app): State<App>,
    Path((ns, wk, store)): Path<(String, String, String)>,
    value: String,
) -> Response {
    match app.write_store(&ns, &wk, &store, value).await {
        Ok(_) => "ok".into_response(),
        Err(e) => e.into_response(),
    }
}

async fn handle_ws_read(
    ws: WebSocketUpgrade,
    State(app): State<App>,
    Path(ns): Path<String>,
) -> Response {
    println!("Handling ws connection to namespace: {}", ns);
    ws.on_upgrade(|socket| app.add_connection(ns, None, socket))
}

async fn handle_ws_write(
    ws: WebSocketUpgrade,
    State(app): State<App>,
    Path((ns, wp)): Path<(String, String)>,
) -> Response {
    println!("Handling ws connection to namespace: {}", ns);
    ws.on_upgrade(move |socket| app.add_connection(ns, Some(wp), socket))
}