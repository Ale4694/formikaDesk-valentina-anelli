use sha2::{Digest, Sha256};
use tauri::Manager;

fn is_valid_format(key: &str) -> bool {
    let parts: Vec<&str> = key.split('-').collect();
    parts.len() == 4
        && parts
            .iter()
            .all(|p| p.len() == 4 && p.chars().all(|c| c.is_ascii_alphanumeric()))
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

pub fn check_license_at_path(app_dir: &std::path::Path) -> bool {
    let license_path = app_dir.join("license.dat");
    let Ok(contents) = std::fs::read_to_string(&license_path) else {
        return false;
    };
    let chiave = contents.trim();
    if !is_valid_format(chiave) {
        return false;
    }
    let Ok(machine_hash) = compute_machine_hash() else {
        return false;
    };
    let key_no_dashes = chiave.replace('-', "").to_lowercase();
    key_no_dashes.len() >= 8
        && machine_hash.len() >= 8
        && key_no_dashes[..8] == machine_hash[..8]
}

#[tauri::command]
pub fn get_machine_id() -> Result<String, String> {
    compute_machine_hash()
}

#[tauri::command]
pub fn check_license(state: tauri::State<'_, crate::LicenseState>) -> bool {
    *state.valid.lock().unwrap()
}

#[tauri::command]
pub fn activate_license(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::LicenseState>,
    chiave: String,
) -> Result<bool, String> {
    if !is_valid_format(&chiave) {
        return Err("Formato chiave non valido. Usa XXXX-XXXX-XXXX-XXXX".to_string());
    }
    let machine_hash = compute_machine_hash()?;
    let key_no_dashes = chiave.replace('-', "").to_lowercase();
    if key_no_dashes.len() < 8
        || machine_hash.len() < 8
        || key_no_dashes[..8] != machine_hash[..8]
    {
        return Err("Chiave non valida per questo dispositivo".to_string());
    }
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    std::fs::write(app_dir.join("license.dat"), &chiave).map_err(|e| e.to_string())?;
    *state.valid.lock().unwrap() = true;
    Ok(true)
}

#[tauri::command]
pub fn deactivate_license(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::LicenseState>,
) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let license_path = app_dir.join("license.dat");
    if license_path.exists() {
        std::fs::remove_file(&license_path).map_err(|e| e.to_string())?;
    }
    *state.valid.lock().unwrap() = false;
    Ok(())
}
