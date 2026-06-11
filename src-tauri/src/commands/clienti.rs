use crate::{AppError, AppState};
use crate::models::{Cliente, NuovoCliente, PaginatedResult};
use tauri::State;

#[tauri::command]
pub async fn get_all_clienti(state: State<'_, AppState>) -> Result<Vec<Cliente>, AppError> {
    let clienti = sqlx::query_as::<_, Cliente>(
        "SELECT * FROM clienti ORDER BY ragione_sociale ASC"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(clienti)
}

#[tauri::command]
pub async fn get_clienti_paginated(
    page: i64,
    page_size: i64,
    search: String,
    state: State<'_, AppState>,
) -> Result<PaginatedResult<Cliente>, AppError> {
    let page_size = page_size.clamp(1, 200);
    let offset = page.max(0) * page_size;
    let pattern = search.trim().to_string();

    if pattern.is_empty() {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM clienti")
            .fetch_one(&state.db).await?;
        let items = sqlx::query_as::<_, Cliente>(
            "SELECT * FROM clienti ORDER BY ragione_sociale ASC LIMIT ? OFFSET ?"
        ).bind(page_size).bind(offset).fetch_all(&state.db).await?;
        Ok(PaginatedResult { items, total, page, page_size })
    } else {
        let like = format!("%{pattern}%");
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM clienti \
             WHERE ragione_sociale LIKE ? OR partita_iva LIKE ? OR telefono LIKE ? OR email LIKE ?"
        ).bind(&like).bind(&like).bind(&like).bind(&like).fetch_one(&state.db).await?;
        let items = sqlx::query_as::<_, Cliente>(
            "SELECT * FROM clienti \
             WHERE ragione_sociale LIKE ? OR partita_iva LIKE ? OR telefono LIKE ? OR email LIKE ? \
             ORDER BY ragione_sociale ASC LIMIT ? OFFSET ?"
        ).bind(&like).bind(&like).bind(&like).bind(&like)
         .bind(page_size).bind(offset).fetch_all(&state.db).await?;
        Ok(PaginatedResult { items, total, page, page_size })
    }
}

#[tauri::command]
pub async fn get_cliente(id: i64, state: State<'_, AppState>) -> Result<Cliente, AppError> {
    let cliente = sqlx::query_as::<_, Cliente>(
        "SELECT * FROM clienti WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("cliente id={id} non trovato")))?;
    Ok(cliente)
}

#[tauri::command]
pub async fn create_cliente(
    cliente: NuovoCliente,
    state: State<'_, AppState>,
) -> Result<Cliente, AppError> {
    if cliente.ragione_sociale.trim().is_empty() {
        return Err(AppError::Validation("ragione_sociale obbligatoria".into()));
    }
    let id = sqlx::query(
        "INSERT INTO clienti (ragione_sociale, partita_iva, codice_fiscale, indirizzo, citta, cap, provincia, telefono, email, note)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&cliente.ragione_sociale)
    .bind(&cliente.partita_iva)
    .bind(&cliente.codice_fiscale)
    .bind(&cliente.indirizzo)
    .bind(&cliente.citta)
    .bind(&cliente.cap)
    .bind(&cliente.provincia)
    .bind(&cliente.telefono)
    .bind(&cliente.email)
    .bind(&cliente.note)
    .execute(&state.db)
    .await?
    .last_insert_rowid();

    get_cliente(id, state).await
}

#[tauri::command]
pub async fn update_cliente(
    id: i64,
    cliente: NuovoCliente,
    state: State<'_, AppState>,
) -> Result<Cliente, AppError> {
    if cliente.ragione_sociale.trim().is_empty() {
        return Err(AppError::Validation("ragione_sociale obbligatoria".into()));
    }
    let rows = sqlx::query(
        "UPDATE clienti SET ragione_sociale=?, partita_iva=?, codice_fiscale=?, indirizzo=?,
         citta=?, cap=?, provincia=?, telefono=?, email=?, note=?,
         updated_at=datetime('now') WHERE id=?"
    )
    .bind(&cliente.ragione_sociale)
    .bind(&cliente.partita_iva)
    .bind(&cliente.codice_fiscale)
    .bind(&cliente.indirizzo)
    .bind(&cliente.citta)
    .bind(&cliente.cap)
    .bind(&cliente.provincia)
    .bind(&cliente.telefono)
    .bind(&cliente.email)
    .bind(&cliente.note)
    .bind(id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("cliente id={id} non trovato")));
    }
    get_cliente(id, state).await
}

#[tauri::command]
pub async fn delete_cliente(id: i64, state: State<'_, AppState>) -> Result<(), AppError> {
    let rows = sqlx::query("DELETE FROM clienti WHERE id=?")
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("cliente id={id} non trovato")));
    }
    Ok(())
}
