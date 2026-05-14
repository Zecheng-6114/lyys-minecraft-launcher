mod manifest;
mod downloader;
mod launcher;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lml", about = "Minimal Minecraft launcher")] 
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ListVersions,
    Download { version: String },
    Launch { version: String, username: String },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Commands::ListVersions => {
            let manifest = manifest::fetch_manifest()?;
            for v in manifest.versions.iter().rev().take(20) {
                println!("{}", v.id);
            }
        }
        Commands::Download { version } => {
            // download client jar
            let vjson = manifest::fetch_version_json(&version)?;
            let client_url = manifest::client_download_url(&vjson).ok_or("client url missing")?;
            let appdata = std::env::var("APPDATA").ok();
            let base = if let Some(a) = appdata { std::path::PathBuf::from(a) } else { dirs::home_dir().ok_or("Cannot determine home dir")? };
            let mut dest = base; dest.push(".minecraft"); dest.push("versions"); dest.push(&version);
            std::fs::create_dir_all(&dest)?;
            let jar = dest.join(format!("{}.jar", version));
            if !jar.exists() { downloader::download_to_path(&client_url, &jar)?; }
            println!("Client saved to {}", jar.display());
        }
        Commands::Launch { version, username } => {
            launcher::build_and_launch(&version, &username)?;
        }
    }
    Ok(())
}