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

    let api_url = config
        .update_endpoint
        .replace("releases/latest/download/latest.json", "releases/latest");

    let client = reqwest::Client::new();
    let resp = client
        .get(&api_url)
        .header("User-Agent", "autoparts-gestionale")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let release: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let tag = release["tag_name"].as_str().unwrap_or("").to_string();
    let notes = release["body"].as_str().unwrap_or("").to_string();

    let assets = release["assets"].as_array().ok_or("no assets")?;
    let exe_asset = assets
        .iter()
        .find(|a| a["name"].as_str().unwrap_or("").ends_with("x64-setup.exe"));

    let download_url = match exe_asset {
        Some(a) => a["browser_download_url"]
            .as_str()
            .unwrap_or("")
            .to_string(),
        None => return Ok(None),
    };

    let remote_version = tag
        .trim_start_matches('v')
        .split('-')
        .next()
        .unwrap_or("")
        .to_string();

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

    if parse_ver(&remote_version) > parse_ver(&current) {
        Ok(Some(ReleaseInfo {
            version: remote_version,
            download_url,
            notes,
        }))
    } else {
        Ok(None)
    }
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
