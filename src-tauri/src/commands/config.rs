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
    pub email: Option<String>,
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
    // Preferisce l'override in app_data_dir (scritto dall'utente) rispetto al bundle
    if let Ok(data_dir) = app.path().app_data_dir() {
        let override_path = data_dir.join("config.json");
        if override_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&override_path) {
                if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                    return Ok(config);
                }
            }
        }
    }
    // Candidati per il config bundled (resource_dir non sempre corretto su Windows)
    let candidates: Vec<std::path::PathBuf> = [
        app.path().resource_dir().ok().map(|d| d.join("config.json")),
        std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("config.json"))),
    ]
    .into_iter()
    .flatten()
    .collect();

    for path in &candidates {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return Ok(config);
            }
        }
    }

    Ok(AppConfig {
        nome_attivita: String::new(),
        intestazione: None,
        tipo: String::new(),
        update_endpoint: "https://raw.githubusercontent.com/Ale4694/formikaDesk-valentina-anelli/valentina-anelli/latest.json".to_string(),
        moduli: Vec::new(),
        vocabolario: HashMap::new(),
    })
}

#[tauri::command]
pub fn get_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    read_config(&app)
}

#[tauri::command]
pub fn salva_intestazione(intestazione: Intestazione, app: tauri::AppHandle) -> Result<AppConfig, String> {
    let mut config = read_config(&app)?;
    config.intestazione = Some(intestazione);

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Impossibile creare directory: {}", e))?;
    let override_path = data_dir.join("config.json");
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Errore serializzazione: {}", e))?;
    std::fs::write(&override_path, json)
        .map_err(|e| format!("Impossibile scrivere config: {}", e))?;

    Ok(config)
}
