use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intestazione {
    pub ragione_sociale: String,
    pub sottotitolo: Option<String>,
    pub indirizzo: Option<String>,
    pub cap_citta: Option<String>,
    pub piva: Option<String>,
    pub cf: Option<String>,
    pub telefono: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub nome_attivita: String,
    pub intestazione: Option<Intestazione>,
    pub tipo: String,
    pub update_endpoint: String,
    pub moduli: Vec<String>,
    pub vocabolario: HashMap<String, String>,
}

pub fn read_config(app: &tauri::AppHandle) -> Result<AppConfig, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    let config_path = resource_dir.join("config.json");
    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Impossibile leggere config.json: {}", e))?;
    serde_json::from_str(&content).map_err(|e| format!("Config non valida: {}", e))
}

#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    read_config(&app)
}
