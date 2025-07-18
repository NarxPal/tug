use axum::{routing::post, Json, Router};
use serde::Deserialize;
use std::fs;
use std::net::SocketAddr;
use uuid::Uuid;

// using local crates
use images::*;

#[derive(Deserialize)]
struct ContainerConfig {
    image: String,
    command: String,
}

async fn create_container(Json(payload): Json<ContainerConfig>) -> &'static str {
    println!(
        "creating container from image {} and command {}",
        payload.image, payload.command
    );

    let container_id = Uuid::new_v4().to_string();
    let rootfs_path = format!(
        "{}/tug/containers/{}/rootfs",
        std::env::var("HOME").unwrap(),
        container_id
    );

    fs::create_dir_all(&rootfs_path).unwrap();

    // creating isolated environment first
    pull_and_extract_ubuntu_image(&container_id).await.unwrap();
    "container created"
}

#[tokio::main]
async fn main() {
    let app: Router = Router::new().route("/containers/create", post(create_container));
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("listening on address {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
