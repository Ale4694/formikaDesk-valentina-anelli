use serde::Serialize;
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;

#[derive(Serialize)]
pub struct UpdateInfo {
    pub version: String,
    pub notes: String,
    pub date: String,
}

#[tauri::command]
pub async fn check_update(app: tauri::AppHandle) -> Result<Option<UpdateInfo>, String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;
    Ok(update.map(|u| UpdateInfo {
        version: u.version.clone(),
        notes: u.body.clone().unwrap_or_default(),
        date: u.date.map(|d| d.to_string()).unwrap_or_default(),
    }))
}

#[tauri::command]
pub async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    let updater = app.updater_builder().build().map_err(|e| e.to_string())?;
    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        let app_clone = app.clone();
        update
            .download_and_install(
                move |downloaded, total| {
                    if let Some(total) = total {
                        let progress = (downloaded as f64 / total as f64 * 100.0) as u32;
                        let _ = app_clone.emit("update-progress", progress);
                    }
                },
                || {},
            )
            .await
            .map_err(|e| e.to_string())?;
        app.restart();
    }
    Ok(())
}
