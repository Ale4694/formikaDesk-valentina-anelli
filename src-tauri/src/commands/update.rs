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
    let config = read_config(&app).map_err(|e| e.to_string())?;

    let client = reqwest::Client::new();
    let resp = client
        .get(&config.update_endpoint)
        .header("User-Agent", "autoparts-gestionale")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let latest: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let remote_version = latest["version"].as_str().unwrap_or("").to_string();
    let notes = latest["notes"].as_str().unwrap_or("").to_string();

    if remote_version.is_empty() {
        return Ok(None);
    }

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

    let base = config.update_endpoint.replace("latest.json", "");
    let exe_name = format!("AutoParts.Gestionale_{}_x64-setup.exe", remote_version);
    let download_url = format!("{}{}", base, exe_name);

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
