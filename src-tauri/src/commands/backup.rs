use crate::{AppError, AppState};
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, FilePath};

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn opt(o: &Option<String>) -> &str {
    o.as_deref().unwrap_or("")
}

#[tauri::command]
pub async fn export_ricambi_csv(state: tauri::State<'_, AppState>) -> Result<String, AppError> {
    let ricambi = sqlx::query_as::<_, crate::models::Ricambio>(
        "SELECT * FROM ricambi ORDER BY descrizione ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut out = String::from(
        "Codice Interno,Codice OEM,Descrizione,Marca,Modello Auto,\
         Anno Da,Anno A,Categoria,Giacenza,Giacenza Minima,\
         Prezzo Acquisto,Prezzo Vendita,IVA %,Posizione,Note\n",
    );

    for r in &ricambi {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{:.2},{:.2},{:.2},{},{}\n",
            csv_escape(&r.codice_interno),
            csv_escape(opt(&r.codice_oem)),
            csv_escape(&r.descrizione),
            csv_escape(opt(&r.marca)),
            csv_escape(opt(&r.modello_auto)),
            r.anno_da.map(|v| v.to_string()).unwrap_or_default(),
            r.anno_a.map(|v| v.to_string()).unwrap_or_default(),
            csv_escape(opt(&r.categoria)),
            r.giacenza,
            r.giacenza_minima,
            r.prezzo_acquisto,
            r.prezzo_vendita,
            r.iva_percentuale,
            csv_escape(opt(&r.posizione)),
            csv_escape(opt(&r.note)),
        ));
    }

    Ok(out)
}

#[tauri::command]
pub async fn export_clienti_csv(state: tauri::State<'_, AppState>) -> Result<String, AppError> {
    let clienti = sqlx::query_as::<_, crate::models::Cliente>(
        "SELECT * FROM clienti ORDER BY ragione_sociale ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut out = String::from(
        "Ragione Sociale,Partita IVA,Codice Fiscale,Indirizzo,\
         Città,CAP,Provincia,Telefono,Email,Note\n",
    );

    for c in &clienti {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            csv_escape(&c.ragione_sociale),
            csv_escape(opt(&c.partita_iva)),
            csv_escape(opt(&c.codice_fiscale)),
            csv_escape(opt(&c.indirizzo)),
            csv_escape(opt(&c.citta)),
            csv_escape(opt(&c.cap)),
            csv_escape(opt(&c.provincia)),
            csv_escape(opt(&c.telefono)),
            csv_escape(opt(&c.email)),
            csv_escape(opt(&c.note)),
        ));
    }

    Ok(out)
}

#[tauri::command]
pub async fn backup_database(app: tauri::AppHandle) -> Result<String, AppError> {
    let state = app.state::<AppState>();
    let db_path = state.db_path.clone();
    let backup_dir = state.backup_dir.clone();

    let save_path = app
        .dialog()
        .file()
        .add_filter("Database SQLite", &["sqlite"])
        .set_directory(&backup_dir)
        .blocking_save_file();

    match save_path {
        Some(FilePath::Path(dest)) => {
            std::fs::copy(&db_path, &dest)
                .map_err(|e| AppError::Internal(format!("Errore copia: {e}")))?;
            Ok(dest.display().to_string())
        }
        _ => Ok(String::new()),
    }
}

#[tauri::command]
pub async fn restore_database(app: tauri::AppHandle) -> Result<String, AppError> {
    let state = app.state::<AppState>();
    let db_path = state.db_path.clone();

    let picked = app
        .dialog()
        .file()
        .add_filter("Database SQLite", &["sqlite"])
        .blocking_pick_file();

    match picked {
        Some(FilePath::Path(src)) => {
            std::fs::copy(&src, &db_path)
                .map_err(|e| AppError::Internal(format!("Errore ripristino: {e}")))?;
            Ok("ripristinato".to_string())
        }
        _ => Ok(String::new()),
    }
}
