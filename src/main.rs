use axum::{response::IntoResponse, routing::get, Router};

pub mod store;
pub mod ws;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port");

    println!("Listening on 3000");

    axum::serve(listener, app).await.expect("server failed");
}

async fn root() -> impl IntoResponse {
    &[20]
}
