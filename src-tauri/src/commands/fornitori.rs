use crate::{AppError, AppState};
use crate::models::{Fornitore, NuovoFornitore};
use tauri::State;

#[tauri::command]
pub async fn update_fornitore(
    id: i64,
    fornitore: NuovoFornitore,
    state: State<'_, AppState>,
) -> Result<Fornitore, AppError> {
    if fornitore.ragione_sociale.trim().is_empty() {
        return Err(AppError::Validation("ragione_sociale obbligatoria".into()));
    }
    let rows = sqlx::query(
        "UPDATE fornitori SET ragione_sociale=?, partita_iva=?, codice_fiscale=?, \
         indirizzo=?, citta=?, cap=?, provincia=?, telefono=?, email=?, note=?, \
         updated_at=datetime('now') WHERE id=?",
    )
    .bind(fornitore.ragione_sociale.trim())
    .bind(&fornitore.partita_iva)
    .bind(&fornitore.codice_fiscale)
    .bind(&fornitore.indirizzo)
    .bind(&fornitore.citta)
    .bind(&fornitore.cap)
    .bind(&fornitore.provincia)
    .bind(&fornitore.telefono)
    .bind(&fornitore.email)
    .bind(&fornitore.note)
    .bind(id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("fornitore id={id} non trovato")));
    }

    let updated = sqlx::query_as::<_, Fornitore>("SELECT * FROM fornitori WHERE id=?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    Ok(updated)
}

#[tauri::command]
pub async fn delete_fornitore(id: i64, state: State<'_, AppState>) -> Result<(), AppError> {
    let n_doc: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM documenti WHERE fornitore_id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    if n_doc > 0 {
        return Err(AppError::Validation(format!(
            "Impossibile eliminare: il fornitore ha {} document{} collegat{}",
            n_doc,
            if n_doc == 1 { "o" } else { "i" },
            if n_doc == 1 { "o" } else { "i" },
        )));
    }

    let n_ord: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ordini_fornitore WHERE fornitore_id = ?",
    )
    .bind(id)
    .fetch_one(&state.db)
    .await?;

    if n_ord > 0 {
        return Err(AppError::Validation(format!(
            "Impossibile eliminare: il fornitore ha {} ordin{} collegat{}",
            n_ord,
            if n_ord == 1 { "e" } else { "i" },
            if n_ord == 1 { "o" } else { "i" },
        )));
    }

    let rows = sqlx::query("DELETE FROM fornitori WHERE id=?")
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("fornitore id={id} non trovato")));
    }
    Ok(())
}

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
