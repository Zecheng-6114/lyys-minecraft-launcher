use serde::Deserialize;
use std::error::Error;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct VersionEntry {
    pub id: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct VersionManifest {
    pub versions: Vec<VersionEntry>,
}

pub fn fetch_manifest() -> Result<VersionManifest, Box<dyn Error>> {
    let url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let resp = reqwest::blocking::get(url)?.text()?;
    let manifest: VersionManifest = serde_json::from_str(&resp)?;
    Ok(manifest)
}

#[derive(Deserialize, Debug)]
pub struct DownloadEntry {
    pub url: String,
    pub path: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadEntry>,
    pub classifiers: Option<HashMap<String, DownloadEntry>>,
}

#[derive(Deserialize, Debug)]
pub struct LibraryEntry {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
}

#[derive(Deserialize, Debug)]
pub struct VersionJson {
    pub downloads: Option<HashMap<String, DownloadEntry>>,
    pub libraries: Option<Vec<LibraryEntry>>,
    pub assetIndex: Option<DownloadEntry>,
    pub assets: Option<String>,
    pub mainClass: Option<String>,
}

pub fn fetch_version_json(version: &str) -> Result<VersionJson, Box<dyn Error>> {
    let manifest = fetch_manifest()?;
    let entry = manifest.versions.iter().find(|v| v.id == version)
        .ok_or_else(|| format!("Version {} not found", version))?;
    let version_json_text = reqwest::blocking::get(&entry.url)?.text()?;
    let vjson: VersionJson = serde_json::from_str(&version_json_text)?;
    Ok(vjson)
}

pub fn client_download_url(vjson: &VersionJson) -> Option<String> {
    vjson.downloads.as_ref()
        .and_then(|m| m.get("client").map(|d| d.url.clone()))
}
