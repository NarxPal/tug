use anyhow;
use bytes::Bytes;
use flate2::read::GzDecoder;
use reqwest::blocking::Client as BlockingClient;
use reqwest::Client;
use serde::Deserialize;
use std::{
    error::Error,
    io::{self, Cursor},
    path::Path,
};
use tar::Archive;

pub mod build;

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    #[serde(rename = "layers")]
    layers: Vec<Layer>,
}

#[derive(Debug, Deserialize)]
struct Layer {
    digest: String,
}

#[derive(Debug, Deserialize)]
struct ManifestList {
    manifests: Vec<PlatformManifest>,
}

#[derive(Debug, Deserialize)]
struct PlatformManifest {
    digest: String,
    mediaType: String,
    platform: Platform,
}

#[derive(Debug, Deserialize)]
struct Platform {
    architecture: String,
    os: String,
}

pub async fn fetch_docker_token() -> Result<String, reqwest::Error> {
    let url = "https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/ubuntu:pull";

    let client = Client::new();
    let resp = client
        .get(url)
        .send()
        .await?
        .json::<TokenResponse>()
        .await?;

    Ok(resp.token)
}

pub async fn save_and_extract_layer(bytes: Bytes, extract_to: &Path) -> Result<(), io::Error> {
    let tar = GzDecoder::new(Cursor::new(bytes));
    let mut archive = Archive::new(tar);
    archive.unpack(extract_to)?;
    Ok(())
}

pub async fn fetch_manifest(container_id: &str) -> Result<(), Box<dyn Error>> {
    let token = fetch_docker_token().await?;

    let manifest_url = "https://registry-1.docker.io/v2/library/ubuntu/manifests/latest";

    let client = Client::new();
    let res = client
        .get(manifest_url)
        .bearer_auth(&token)
        .header(
            "Accept",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .send()
        .await?;

    let raw_img = res.text().await?;
    println!("raw img {}", raw_img);

    // raw_img contains manifest list
    let manifest_list: ManifestList = serde_json::from_str(&raw_img)?;

    let selected = manifest_list
        .manifests
        .iter()
        .find(|m| m.platform.os == "linux" && m.platform.architecture == "amd64")
        .ok_or_else(|| anyhow::anyhow!("No compatible platform found"))?;

    let digest_url = format!(
        "https://registry-1.docker.io/v2/library/ubuntu/manifests/{}",
        selected.digest
    );

    let manifest_res = client
        .get(&digest_url)
        .bearer_auth(&token)
        .header(
            "Accept",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .send()
        .await?;

    let manifest_str = manifest_res.text().await?;
    let manifest: Manifest = serde_json::from_str(&manifest_str)?;

    for layer in manifest.layers {
        let url = format!(
            "https://registry-1.docker.io/v2/library/ubuntu/blobs/{}",
            layer.digest
        );

        let layer_res = client.get(&url).bearer_auth(&token).send().await?;
        let bytes = layer_res.bytes().await?; // .bytes will give the raw data for the .tar.gz file

        let home = std::env::var("HOME").unwrap();
        let extract_path_str = format!("{}/tug/containers/{}/rootfs", home, container_id);

        let extract_path = Path::new(&extract_path_str);

        save_and_extract_layer(bytes, extract_path).await?;
    }

    Ok(())
}

pub async fn pull_and_extract_ubuntu_image(
    container_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    fetch_manifest(container_id).await?;
    Ok(())
}

fn parse_image(image: &str) -> (String, String) {
    let parts: Vec<&str> = image.split(':').collect();
    let repo = parts[0].to_string();
    let tag = parts.get(1).unwrap_or(&"latest").to_string();
    (repo, tag) // returning image name, version
}

pub fn pull_and_extract_image(image: &str, dest: &Path) {
    let (img_name, version) = parse_image(image);
    let client = BlockingClient::new();

    let token_url = format!(
        "https://auth.docker.io/token?service=registry.docker.io&scope=repository:library/{}:pull",
        img_name
    );

    let token_resp: serde_json::Value = client.get(&token_url).send().unwrap().json().unwrap();

    println!("token res from dockerhub {}", token_resp);

    let token = token_resp["token"].as_str().unwrap();

    let manifest_url = format!(
        "https://registry-1.docker.io/v2/library/{}/manifests/{}",
        img_name, version
    );
    let manifest_resp = client
        .get(&manifest_url)
        .header("Authorization", format!("Bearer {}", token))
        .header(
            "Accept",
            "application/vnd.docker.distribution.manifest.v2+json",
        )
        .send()
        .unwrap();

    let manifeset_json: serde_json::Value = manifest_resp.json().unwrap();

    println!("manifest: {}", manifeset_json);

    let layers = manifeset_json["layers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["digest"].as_str().unwrap().to_string()) // using digest from each object inside layers since digest is used to download layer blob
        .collect::<Vec<_>>();

    for digest in layers {
        println!("Pulling layer {}", digest);
        let layer_url = format!(
            "https://registry-1.docker.io/v2/library/{}/blobs/{}",
            img_name, digest
        );
        let layer_resp = client
            .get(&layer_url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .unwrap();

        let mut decoder = GzDecoder::new(layer_resp);
        let mut archive = Archive::new(decoder);
        archive.unpack(dest).unwrap(); // unpack the file
    }
}
