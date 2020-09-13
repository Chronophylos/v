use anyhow::{ensure, Result};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Image {
    pub link: String,
}

#[derive(Debug, Deserialize)]
struct AlbumImages {
    pub data: Vec<Image>,
    pub status: u8,
}

pub fn get_album_images(client_id: &str, album_hash: &str) -> Result<Vec<String>> {
    let client = Client::new();
    let resp = client
        .get(&format!(
            "https://api.imgur.com/3/album/{}/images",
            album_hash
        ))
        .header("Authorization", format!("Client-ID {}", client_id))
        .send()?;

    ensure!(
        resp.status().is_success(),
        format!("Imgur said: {}", resp.status())
    );

    let json = resp.json::<AlbumImages>()?;

    ensure!(
        json.status == 200,
        format!("Imgur API status is {}", json.status)
    );

    Ok(json.data.iter().map(|image| image.link.clone()).collect())
}
