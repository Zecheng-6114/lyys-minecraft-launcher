use std::path::PathBuf;
use std::collections::HashMap;
use std::process::Command;
use std::fs::File;
use std::io;

use crate::manifest;

fn appdata_base() -> PathBuf {
    if let Ok(a) = std::env::var("APPDATA") { PathBuf::from(a) }
    else { dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")) }
}

fn libraries_dir() -> PathBuf {
    let mut p = appdata_base();
    p.push(".minecraft");
    p.push("libraries");
    p
}

fn versions_dir() -> PathBuf {
    let mut p = appdata_base();
    p.push(".minecraft");
    p.push("versions");
    p
}

fn assets_dir() -> PathBuf {
    let mut p = appdata_base();
    p.push(".minecraft");
    p.push("assets");
    p
}

fn find_java() -> Result<String, Box<dyn std::error::Error>> {
    // Try env var first
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let java = PathBuf::from(&java_home).join("bin").join("java.exe");
        if java.exists() { return Ok(java.to_string_lossy().to_string()); }
    }
    // Try JAVA_PATH
    if let Ok(java_path) = std::env::var("JAVA_PATH") {
        if PathBuf::from(&java_path).exists() { return Ok(java_path); }
    }
    // Try common paths
    for path_str in &[
        "D:\\Game\\Minecraft\\Java\\zulu21.50.19-ca-fx-jdk21.0.11-win_x64\\bin\\java.exe",
        "D:\\Game\\Minecraft\\Java\\zulu17.64.17-ca-fx-jdk17.0.18-win_x64\\bin\\java.exe",
    ] {
        if PathBuf::from(path_str).exists() { return Ok(path_str.to_string()); }
    }
    // Try system PATH
    if Command::new("java").arg("--version").output().is_ok() {
        return Ok("java".to_string());
    }
    Err("Cannot find Java executable. Set JAVA_HOME or JAVA_PATH env var.".into())
}

pub fn ensure_library_artifact(download: &crate::manifest::DownloadEntry) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // If download.path is provided, use it under libraries_dir, else infer from URL
    let libdir = libraries_dir();
    if let Some(p) = &download.path {
        let mut dest = libdir.clone();
        for seg in p.split('/') { dest.push(seg); }
        if dest.exists() { return Ok(dest); }
        if let Some(parent) = dest.parent() { std::fs::create_dir_all(parent)?; }
        crate::downloader::download_to_path(&download.url, &dest)?;
        return Ok(dest);
    }
    // fallback: use URL last segment
    let url = &download.url;
    let filename = url.split('/').last().ok_or("bad url")?;
    let mut dest = libdir;
    dest.push(filename);
    if !dest.exists() {
        if let Some(parent) = dest.parent() { std::fs::create_dir_all(parent)?; }
        crate::downloader::download_to_path(&download.url, &dest)?;
    }
    Ok(dest)
}

pub fn download_libraries_and_natives(version: &str) -> Result<(PathBuf, Vec<PathBuf>, Option<PathBuf>), Box<dyn std::error::Error>> {
    let vjson = manifest::fetch_version_json(version)?;
    // client jar
    let client_url = manifest::client_download_url(&vjson).ok_or("client url missing")?;
    let mut client_dest = versions_dir();
    client_dest.push(version);
    std::fs::create_dir_all(&client_dest)?;
    let client_jar = client_dest.join(format!("{}.jar", version));
    if !client_jar.exists() { crate::downloader::download_to_path(&client_url, &client_jar)?; }

    // libraries
    let mut classpath_jars: Vec<PathBuf> = Vec::new();
    if let Some(libs) = vjson.libraries.as_ref() {
        for lib in libs {
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let p = ensure_library_artifact(artifact)?;
                    classpath_jars.push(p);
                }
            }
        }
    }

    // natives
    let mut natives_dir: Option<PathBuf> = None;
    if let Some(libs) = vjson.libraries.as_ref() {
        for lib in libs {
            if let Some(downloads) = &lib.downloads {
                if let Some(classifiers) = &downloads.classifiers {
                    // look for windows natives
                    for (k, entry) in classifiers {
                        if k.contains("natives-windows") {
                            let path = ensure_library_artifact(entry)?;
                            // extract zip to natives/<version>
                            let mut nroot = appdata_base();
                            nroot.push(".minecraft"); nroot.push("natives"); nroot.push(version);
                            std::fs::create_dir_all(&nroot)?;
                            // open zip and extract
                            let file = File::open(&path)?;
                            let mut archive = zip::ZipArchive::new(file)?;
                            for i in 0..archive.len() {
                                let mut f = archive.by_index(i)?;
                                let outpath = nroot.join(f.name());
                                if f.is_dir() { std::fs::create_dir_all(&outpath)?; }
                                else {
                                    if let Some(parent) = outpath.parent() { std::fs::create_dir_all(parent)?; }
                                    let mut outfile = File::create(&outpath)?;
                                    io::copy(&mut f, &mut outfile)?;
                                }
                            }
                            natives_dir = Some(nroot);
                        }
                    }
                }
            }
        }
    }

    Ok((client_jar, classpath_jars, natives_dir))
}

pub fn download_assets(version: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let vjson = manifest::fetch_version_json(version)?;
    let asset_index = vjson.assetIndex.ok_or("assetIndex missing")?;
    let assets_root = assets_dir();
    let indexes_dir = assets_root.join("indexes");
    let objects_dir = assets_root.join("objects");
    std::fs::create_dir_all(&indexes_dir)?;
    std::fs::create_dir_all(&objects_dir)?;
    // download index json
    let index_path = indexes_dir.join(format!("{}.json", asset_index.path.as_ref().map(|s| s.split('/').last().unwrap_or(s)).unwrap_or("index".into())));
    if !index_path.exists() { crate::downloader::download_to_path(&asset_index.url, &index_path)?; }
    let index_txt = std::fs::read_to_string(&index_path)?;
    #[derive(serde::Deserialize)] struct AssetIndex { objects: HashMap<String, serde_json::Value> }
    let idx: AssetIndex = serde_json::from_str(&index_txt)?;
    for (_name, obj) in idx.objects {
        let hash = obj.get("hash").and_then(|h| h.as_str()).ok_or("bad asset entry")?.to_string();
        let sub = &hash[0..2];
        let mut objpath = objects_dir.join(sub); objpath.push(&hash);
        if !objpath.exists() {
            std::fs::create_dir_all(objpath.parent().unwrap())?;
            let url = format!("https://resources.download.minecraft.net/{}/{}", sub, hash);
            crate::downloader::download_to_path(&url, &objpath)?;
        }
    }
    Ok(assets_root)
}

pub fn build_and_launch(version: &str, username: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Downloading libraries and natives...");
    let (client_jar, mut jars, natives_dir) = download_libraries_and_natives(version)?;
    jars.push(client_jar.clone());
    let cp = jars.iter().map(|p| p.to_string_lossy()).collect::<Vec<_>>().join(";");

    println!("Downloading assets...");
    let assets = download_assets(version).ok();

    let java_exe = find_java()?;
    println!("Using Java: {}", java_exe);

    let mut args = Vec::new();
    if let Some(nat) = natives_dir {
        args.push(format!("-Djava.library.path={}", nat.display()));
    }
    args.push("-cp".into());
    args.push(cp);
    let main_class = {
        let vjson = manifest::fetch_version_json(version)?;
        vjson.mainClass.unwrap_or_else(|| "net.minecraft.client.main.Main".into())
    };
    args.push(main_class);
    args.push(format!("--username")); args.push(username.into());
    args.push(format!("--version")); args.push(version.into());
    let mut game_dir = versions_dir(); game_dir.push(version);
    args.push(format!("--gameDir")); args.push(game_dir.to_string_lossy().to_string());
    if let Some(a) = assets { args.push("--assetsDir".into()); args.push(a.to_string_lossy().to_string()); }
    args.push("--uuid".into()); args.push(username.into());
    args.push("--accessToken".into()); args.push("0".into());
    args.push("--userType".into()); args.push("legacy".into());

    println!("Launching Minecraft...");
    let mut cmd = Command::new(&java_exe);
    for a in args { cmd.arg(a); }
    cmd.spawn()?.wait()?;
    Ok(())
}
