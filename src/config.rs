use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashSet, fs::File, io::Read, path::Path};

#[derive(Debug, Deserialize)]
pub struct ImgurConfig {
    #[serde(rename = "client-id")]
    pub client_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub imgur: ImgurConfig,
    #[serde(rename = "allowed-domains")]
    pub allowed_domains: HashSet<String>,
}

impl Config {
    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let mut f = File::open(path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        Ok(toml::de::from_slice(buf.as_slice())?)
    }
}
