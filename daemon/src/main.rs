use axum::{routing::post, Json, Router};
use images::build::build_from_instructions;
use serde::Deserialize;
use std::fs;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::os::unix::net::UnixListener;
use uuid::Uuid;

// using local crates
use cli::TugRequest;
use images::*;
use tugfile::instruction::{self, Instruction};

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
    let socket_path = "/run/tugd.sock";
    let _ = fs::remove_file(socket_path); // to clean fs in case of crash

    let listener = UnixListener::bind(socket_path).expect("Failed to bind socket");

    println!("tugd running at {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let mut buf = [0u8; 8192]; // setting byte to 0
                let n = s.read(&mut buf).unwrap(); // reading received data
                let req_str = String::from_utf8_lossy(&buf[..n]);

                println!("Received: {}", req_str);

                let parsed: Result<TugRequest, _> = serde_json::from_str(&req_str); // deserialize json into struct
                match parsed {
                    Ok(req) => match req.command.as_str() {
                        "build" => {
                            if let Some(inst) = req.instructions {
                                build_from_instructions(inst);
                                s.write_all(b"Build complete").unwrap();
                            }
                        }
                        // "pull" => {
                        //     if let Some(image) = req.image {
                        //         pull_image(&image);
                        //         s.write_all(b"Pulled image").unwrap();
                        //     }
                        // }
                        _ => s.write_all(b"Unknown command").unwrap(),
                    },
                    Err(e) => {
                        eprintln!("invalid request: {}", e);
                        s.write_all(b"invalid request").unwrap();
                    }
                }
                s.write_all(b"OK from tugd").unwrap();
            }
            Err(e) => eprintln!("Socket error: {}", e),
        }
    }
}
