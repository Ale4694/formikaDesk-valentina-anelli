use tauri::command;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub download_url: String,
    pub notes: String,
}

#[command]
pub async fn check_update_custom(app: tauri::AppHandle) -> Result<Option<ReleaseInfo>, String> {
    use crate::commands::config::read_config;
    let api_url = match read_config(&app) {
        Ok(config) => config.update_endpoint,
        Err(_) => "https://github.com/Ale4694/formikaDesk-valentina-anelli/releases/latest/download/latest.json".to_string(),
    };

    let client = reqwest::Client::new();
    let resp = client
        .get(&api_url)
        .header("User-Agent", "autoparts-gestionale")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        return Ok(None);
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let remote_version = match body["version"].as_str() {
        Some(v) => v.to_string(),
        None => return Ok(None),
    };
    let notes = body["notes"].as_str().unwrap_or("").to_string();

    let current = app.package_info().version.to_string();

    let parse_ver = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v.split('.').filter_map(|x| x.parse().ok()).collect();
        (
            parts.get(0).copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };

    if parse_ver(&remote_version) <= parse_ver(&current) {
        return Ok(None);
    }

    let download_url = match body["download_url"].as_str() {
        Some(u) => u.to_string(),
        None => return Ok(None),
    };

    Ok(Some(ReleaseInfo {
        version: remote_version,
        download_url,
        notes,
    }))
}

#[command]
pub async fn download_and_install_update(
    download_url: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use std::process::Command;

    let client = reqwest::Client::new();
    let resp = client
        .get(&download_url)
        .header("User-Agent", "autoparts-gestionale")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;

    let tmp_path = std::env::temp_dir().join("autoparts-update-setup.exe");
    std::fs::write(&tmp_path, &bytes).map_err(|e| e.to_string())?;

    Command::new(&tmp_path)
        .spawn()
        .map_err(|e| e.to_string())?;

    app.exit(0);
    Ok(())
}
