use anyhow;
use bytes::Bytes;
use flate2::read::GzDecoder;
use reqwest::Client;
use serde::Deserialize;
use std::{
    error::Error,
    io::{self, Cursor},
    path::Path,
};
use tar::Archive;

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(rename = "layers")]
    layers: Vec<Layer>,
}

#[derive(Deserialize)]
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
