use crate::{AppError, AppState};
use std::collections::HashMap;
use tauri::State;

#[derive(sqlx::FromRow)]
struct ImpostazioneRow {
    chiave: String,
    valore: String,
}

#[tauri::command]
pub async fn get_impostazioni(state: State<'_, AppState>) -> Result<HashMap<String, String>, AppError> {
    let rows = sqlx::query_as::<_, ImpostazioneRow>("SELECT chiave, valore FROM impostazioni")
        .fetch_all(&state.db)
        .await?;

    let map = rows.into_iter().map(|r| (r.chiave, r.valore)).collect();
    Ok(map)
}

#[tauri::command]
pub async fn save_impostazioni(
    impostazioni: HashMap<String, String>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    for (chiave, valore) in impostazioni {
        sqlx::query(
            "INSERT INTO impostazioni (chiave, valore) VALUES (?, ?)
             ON CONFLICT(chiave) DO UPDATE SET valore=excluded.valore",
        )
        .bind(&chiave)
        .bind(&valore)
        .execute(&state.db)
        .await?;
    }
    Ok(())
}
