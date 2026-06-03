use crate::{AppError, AppState};
use crate::models::{Fornitore, NuovoFornitore};
use tauri::State;

#[tauri::command]
pub async fn get_all_fornitori(state: State<'_, AppState>) -> Result<Vec<Fornitore>, AppError> {
    let fornitori = sqlx::query_as::<_, Fornitore>(
        "SELECT * FROM fornitori ORDER BY ragione_sociale ASC"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(fornitori)
}

#[tauri::command]
pub async fn create_fornitore(
    fornitore: NuovoFornitore,
    state: State<'_, AppState>,
) -> Result<Fornitore, AppError> {
    if fornitore.ragione_sociale.trim().is_empty() {
        return Err(AppError::Validation("ragione_sociale obbligatoria".into()));
    }
    let id = sqlx::query(
        "INSERT INTO fornitori (ragione_sociale, partita_iva, codice_fiscale, indirizzo, citta, cap, provincia, telefono, email, note)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&fornitore.ragione_sociale)
    .bind(&fornitore.partita_iva)
    .bind(&fornitore.codice_fiscale)
    .bind(&fornitore.indirizzo)
    .bind(&fornitore.citta)
    .bind(&fornitore.cap)
    .bind(&fornitore.provincia)
    .bind(&fornitore.telefono)
    .bind(&fornitore.email)
    .bind(&fornitore.note)
    .execute(&state.db)
    .await?
    .last_insert_rowid();

    let fornitore = sqlx::query_as::<_, Fornitore>("SELECT * FROM fornitori WHERE id=?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    Ok(fornitore)
}
