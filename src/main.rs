use std::net::{IpAddr, Ipv4Addr};

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(root))
        .route("/json", get(get_ip_json));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

fn get_ip() -> IpAddr {
    IpAddr::V4(Ipv4Addr::LOCALHOST)
}

// basic handler that responds with a static string
async fn root() -> String {
    tracing::trace!("Get request: /");
    get_ip().to_string()
}

#[derive(Serialize)]
struct Ip {
    ip: IpAddr,
}

async fn get_ip_json() -> impl IntoResponse {
    let ip = Ip { ip: get_ip() };
    (StatusCode::CREATED, Json(ip))
}
