use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub fn download_to_path(url: &str, dest: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut resp = reqwest::blocking::get(url)?;
    if !resp.status().is_success() {
        return Err(format!("Download failed: {}", resp.status()).into());
    }

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut out = File::create(dest)?;
    resp.copy_to(&mut out)?;
    Ok(())
}
