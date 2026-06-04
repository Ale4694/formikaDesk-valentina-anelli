use chrono::{Duration, Local, NaiveDate};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::Manager;

#[derive(Debug, Clone, Serialize)]
pub struct LicenseInfo {
    pub valid: bool,
    pub tipo: String,
    pub scadenza: Option<String>,
    pub giorni_rimanenti: Option<i64>,
}

impl LicenseInfo {
    fn none() -> Self {
        LicenseInfo { valid: false, tipo: "none".into(), scadenza: None, giorni_rimanenti: None }
    }
    fn permanent() -> Self {
        LicenseInfo { valid: true, tipo: "permanent".into(), scadenza: None, giorni_rimanenti: None }
    }
    fn demo(scadenza: String, giorni: i64) -> Self {
        LicenseInfo { valid: giorni >= 0, tipo: "demo".into(), scadenza: Some(scadenza), giorni_rimanenti: Some(giorni) }
    }
}

fn is_valid_format(key: &str) -> bool {
    let parts: Vec<&str> = key.split('-').collect();
    parts.len() == 4
        && parts.iter().all(|p| p.len() == 4 && p.chars().all(|c| c.is_ascii_alphanumeric()))
}

pub fn compute_machine_hash() -> Result<String, String> {
    let raw = read_raw_machine_id()?;
    let prefix: String = raw.chars().take(16).collect();
    let hash = Sha256::digest(prefix.as_bytes());
    Ok(format!("{:x}", hash))
}

#[cfg(target_os = "linux")]
fn read_raw_machine_id() -> Result<String, String> {
    std::fs::read_to_string("/etc/machine-id")
        .map(|s| s.trim().to_string())
        .map_err(|e| format!("Impossibile leggere machine-id: {e}"))
}

#[cfg(target_os = "windows")]
fn read_raw_machine_id() -> Result<String, String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let crypto = hklm
        .open_subkey(r"SOFTWARE\Microsoft\Cryptography")
        .map_err(|e| format!("Errore registry: {e}"))?;
    let guid: String = crypto
        .get_value("MachineGuid")
        .map_err(|e| format!("MachineGuid non trovato: {e}"))?;
    Ok(guid)
}

#[cfg(target_os = "macos")]
fn read_raw_machine_id() -> Result<String, String> {
    let output = std::process::Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .map_err(|e| format!("ioreg error: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("IOPlatformUUID") {
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                let val = parts[1].trim().trim_matches('"');
                return Ok(val.to_string());
            }
        }
    }
    Err("IOPlatformUUID non trovato".to_string())
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn read_raw_machine_id() -> Result<String, String> {
    Err("Piattaforma non supportata".to_string())
}

fn validate_permanent_key(chiave: &str, machine_hash: &str) -> bool {
    let key_no_dashes = chiave.replace('-', "").to_lowercase();
    key_no_dashes.len() >= 8
        && machine_hash.len() >= 8
        && key_no_dashes[..8] == machine_hash[..8]
}

fn parse_demo_entry(rest: &str) -> LicenseInfo {
    // rest = "MMYY:YYYY-MM-DD"
    let parts: Vec<&str> = rest.splitn(2, ':').collect();
    if parts.len() != 2 {
        return LicenseInfo::none();
    }
    let Ok(activation) = NaiveDate::parse_from_str(parts[1], "%Y-%m-%d") else {
        return LicenseInfo::none();
    };
    let expiry = activation + Duration::days(15);
    let today = Local::now().date_naive();
    let giorni = (expiry - today).num_days();
    let scadenza = expiry.format("%d/%m/%Y").to_string();
    LicenseInfo::demo(scadenza, giorni)
}

pub fn check_license_at_path(app_dir: &std::path::Path) -> LicenseInfo {
    let license_path = app_dir.join("license.dat");
    let Ok(contents) = std::fs::read_to_string(&license_path) else {
        return LicenseInfo::none();
    };
    let contents = contents.trim();
    let Ok(machine_hash) = compute_machine_hash() else {
        return LicenseInfo::none();
    };

    if let Some(rest) = contents.strip_prefix("permanent:") {
        if validate_permanent_key(rest, &machine_hash) {
            LicenseInfo::permanent()
        } else {
            LicenseInfo::none()
        }
    } else if let Some(rest) = contents.strip_prefix("demo:") {
        parse_demo_entry(rest)
    } else {
        // Legacy: raw permanent key
        if is_valid_format(contents) && validate_permanent_key(contents, &machine_hash) {
            LicenseInfo::permanent()
        } else {
            LicenseInfo::none()
        }
    }
}

#[tauri::command]
pub fn get_machine_id() -> Result<String, String> {
    compute_machine_hash()
}

#[tauri::command]
pub fn check_license(state: tauri::State<'_, crate::LicenseState>) -> LicenseInfo {
    state.info.lock().unwrap().clone()
}

#[tauri::command]
pub fn activate_license(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::LicenseState>,
    chiave: String,
) -> Result<LicenseInfo, String> {
    let machine_hash = compute_machine_hash()?;
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    if chiave.starts_with("DEMO-") {
        // Formato: DEMO-XXXX-XXXX-MMYY
        let parts: Vec<&str> = chiave.split('-').collect();
        if parts.len() != 4
            || parts[1].len() != 4
            || parts[2].len() != 4
            || parts[3].len() != 4
        {
            return Err("Formato chiave demo non valido. Usa DEMO-XXXX-XXXX-MMYY".into());
        }
        let hash_part = format!("{}{}", parts[1], parts[2]).to_lowercase();
        if machine_hash.len() < 8 || hash_part != machine_hash[..8] {
            return Err("Chiave demo non valida per questo dispositivo".into());
        }
        let mmyy = parts[3];
        let today = Local::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();
        std::fs::write(app_dir.join("license.dat"), format!("demo:{mmyy}:{today_str}"))
            .map_err(|e| e.to_string())?;
        let expiry = today + Duration::days(15);
        let info = LicenseInfo::demo(expiry.format("%d/%m/%Y").to_string(), 15);
        *state.info.lock().unwrap() = info.clone();
        Ok(info)
    } else {
        if !is_valid_format(&chiave) {
            return Err("Formato chiave non valido. Usa XXXX-XXXX-XXXX-XXXX".into());
        }
        if !validate_permanent_key(&chiave, &machine_hash) {
            return Err("Chiave non valida per questo dispositivo".into());
        }
        std::fs::write(app_dir.join("license.dat"), format!("permanent:{chiave}"))
            .map_err(|e| e.to_string())?;
        let info = LicenseInfo::permanent();
        *state.info.lock().unwrap() = info.clone();
        Ok(info)
    }
}

#[tauri::command]
pub fn deactivate_license(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::LicenseState>,
) -> Result<(), String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let license_path = app_dir.join("license.dat");
    if license_path.exists() {
        std::fs::remove_file(&license_path).map_err(|e| e.to_string())?;
    }
    *state.info.lock().unwrap() = LicenseInfo::none();
    Ok(())
}
