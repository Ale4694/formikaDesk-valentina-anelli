use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;
use tauri::Manager;

mod commands;
mod db;
mod error;
mod models;

pub use error::AppError;

pub struct AppState {
    pub db: SqlitePool,
    pub db_path: std::path::PathBuf,
    pub fatture_dir: std::path::PathBuf,
    pub backup_dir: std::path::PathBuf,
}

pub struct LicenseState {
    pub info: std::sync::Mutex<commands::license::LicenseInfo>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Percorso legacy (AppData) — usato solo per licenza e migrazione DB
            let legacy_dir = app
                .path()
                .app_data_dir()
                .expect("impossibile ottenere app_data_dir");
            std::fs::create_dir_all(&legacy_dir)
                .expect("impossibile creare la directory dati legacy");

            let license_info = commands::license::check_license_at_path(&legacy_dir);
            app.manage(LicenseState {
                info: std::sync::Mutex::new(license_info),
            });

            // Nuova struttura in Documenti/FormikaDesk/
            let doc_dir = app
                .path()
                .document_dir()
                .expect("impossibile ottenere la cartella Documenti");
            let formika_dir = doc_dir.join("FormikaDesk");
            let db_dir     = formika_dir.join("database");
            let fatture_dir = formika_dir.join("fatture");
            let backup_dir  = formika_dir.join("backup");

            std::fs::create_dir_all(&db_dir)
                .expect("impossibile creare FormikaDesk/database");
            std::fs::create_dir_all(&fatture_dir)
                .expect("impossibile creare FormikaDesk/fatture");
            std::fs::create_dir_all(&backup_dir)
                .expect("impossibile creare FormikaDesk/backup");

            let db_path = db_dir.join("autoparts.sqlite");

            // Migrazione automatica: se esiste il DB in AppData e non esiste ancora il nuovo, copialo
            let legacy_db = legacy_dir.join("autoparts.sqlite");
            if legacy_db.exists() && !db_path.exists() {
                std::fs::copy(&legacy_db, &db_path)
                    .expect("migrazione database da AppData a Documenti/FormikaDesk fallita");
                log::info!("Database migrato da {:?} a {:?}", legacy_db, db_path);
            }

            let db_url = format!("sqlite:{}", db_path.display());

            let pool = tauri::async_runtime::block_on(async {
                let options = SqliteConnectOptions::from_str(&db_url)
                    .expect("stringa connessione non valida")
                    .create_if_missing(true);

                let pool = SqlitePool::connect_with(options)
                    .await
                    .expect("connessione DB fallita");

                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .expect("migrazione DB fallita");

                pool
            });

            app.manage(AppState { db: pool, db_path, fatture_dir, backup_dir });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::clienti::get_all_clienti,
            commands::clienti::get_clienti_paginated,
            commands::clienti::get_cliente,
            commands::clienti::create_cliente,
            commands::clienti::update_cliente,
            commands::clienti::delete_cliente,
            commands::ricambi::get_all_ricambi,
            commands::ricambi::get_ricambi_paginated,
            commands::ricambi::get_ricambio,
            commands::ricambi::search_ricambi,
            commands::ricambi::create_ricambio,
            commands::ricambi::update_ricambio,
            commands::ricambi::delete_ricambio,
            commands::ricambi::aggiorna_giacenza,
            commands::ricambi::carico_rapido_ricambio,
            commands::fornitori::get_all_fornitori,
            commands::fornitori::create_fornitore,
            commands::documenti::get_all_documenti,
            commands::documenti::get_documenti_paginated,
            commands::documenti::get_documento,
            commands::documenti::create_documento,
            commands::documenti::update_stato_documento,
            commands::documenti::update_documento,
            commands::documenti::delete_documento,
            commands::documenti::get_storico_articoli_cliente,
            commands::dashboard::get_dashboard_stats,
            commands::ricerca::search_global,
            commands::backup::export_ricambi_csv,
            commands::backup::export_clienti_csv,
            commands::backup::backup_database,
            commands::backup::restore_database,
            commands::license::get_machine_id,
            commands::license::check_license,
            commands::license::activate_license,
            commands::license::deactivate_license,
            commands::veicoli::get_veicoli_cliente,
            commands::veicoli::create_veicolo,
            commands::veicoli::update_veicolo,
            commands::veicoli::delete_veicolo,
            commands::ordini::get_all_ordini,
            commands::ordini::create_ordine_fornitore,
            commands::ordini::update_stato_ordine,
            commands::report::get_report_mensile,
            commands::report::get_scadenzario,
            commands::cassa::get_all_scontrini,
            commands::cassa::get_scontrino,
            commands::cassa::cerca_ricambio_barcode,
            commands::cassa::crea_scontrino,
            commands::cassa::annulla_scontrino,
            commands::impostazioni::get_impostazioni,
            commands::impostazioni::save_impostazioni,
            commands::fattura_pa::genera_fattura_pa,
            commands::config::get_config,
            commands::config::salva_intestazione,
            commands::update::check_update,
            commands::update::install_update,
        ])
        .run(tauri::generate_context!())
        .expect("errore durante l'avvio di Tauri");
}
