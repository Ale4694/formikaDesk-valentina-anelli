use crate::{AppError, AppState};
use crate::models::{NuovoRicambio, Ricambio, RicambioResult};
use tauri::State;

#[tauri::command]
pub async fn get_all_ricambi(state: State<'_, AppState>) -> Result<Vec<Ricambio>, AppError> {
    let ricambi = sqlx::query_as::<_, Ricambio>(
        "SELECT * FROM ricambi ORDER BY descrizione ASC"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(ricambi)
}

#[tauri::command]
pub async fn get_ricambio(id: i64, state: State<'_, AppState>) -> Result<Ricambio, AppError> {
    let ricambio = sqlx::query_as::<_, Ricambio>("SELECT * FROM ricambi WHERE id=?")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("ricambio id={id} non trovato")))?;
    Ok(ricambio)
}

#[tauri::command]
pub async fn search_ricambi(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<Ricambio>, AppError> {
    let pattern = format!("%{}%", query.trim());
    let ricambi = sqlx::query_as::<_, Ricambio>(
        "SELECT * FROM ricambi
         WHERE descrizione LIKE ? OR codice_interno LIKE ? OR codice_oem LIKE ? OR marca LIKE ?
         ORDER BY descrizione ASC
         LIMIT 100"
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&state.db)
    .await?;
    Ok(ricambi)
}

#[tauri::command]
pub async fn create_ricambio(
    ricambio: NuovoRicambio,
    state: State<'_, AppState>,
) -> Result<Ricambio, AppError> {
    if ricambio.codice_interno.trim().is_empty() {
        return Err(AppError::Validation("codice_interno obbligatorio".into()));
    }
    if ricambio.descrizione.trim().is_empty() {
        return Err(AppError::Validation("descrizione obbligatoria".into()));
    }
    let id = sqlx::query(
        "INSERT INTO ricambi (codice_interno, codice_oem, descrizione, marca, modello_auto,
         anno_da, anno_a, categoria, fornitore_id, giacenza, giacenza_minima,
         prezzo_acquisto, prezzo_vendita, iva_percentuale, posizione, note)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&ricambio.codice_interno)
    .bind(&ricambio.codice_oem)
    .bind(&ricambio.descrizione)
    .bind(&ricambio.marca)
    .bind(&ricambio.modello_auto)
    .bind(ricambio.anno_da)
    .bind(ricambio.anno_a)
    .bind(&ricambio.categoria)
    .bind(ricambio.fornitore_id)
    .bind(ricambio.giacenza)
    .bind(ricambio.giacenza_minima)
    .bind(ricambio.prezzo_acquisto)
    .bind(ricambio.prezzo_vendita)
    .bind(ricambio.iva_percentuale)
    .bind(&ricambio.posizione)
    .bind(&ricambio.note)
    .execute(&state.db)
    .await?
    .last_insert_rowid();

    get_ricambio(id, state).await
}

#[tauri::command]
pub async fn update_ricambio(
    id: i64,
    ricambio: NuovoRicambio,
    state: State<'_, AppState>,
) -> Result<Ricambio, AppError> {
    if ricambio.descrizione.trim().is_empty() {
        return Err(AppError::Validation("descrizione obbligatoria".into()));
    }
    let rows = sqlx::query(
        "UPDATE ricambi SET codice_interno=?, codice_oem=?, descrizione=?, marca=?,
         modello_auto=?, anno_da=?, anno_a=?, categoria=?, fornitore_id=?,
         giacenza=?, giacenza_minima=?, prezzo_acquisto=?, prezzo_vendita=?,
         iva_percentuale=?, posizione=?, note=?, updated_at=datetime('now')
         WHERE id=?"
    )
    .bind(&ricambio.codice_interno)
    .bind(&ricambio.codice_oem)
    .bind(&ricambio.descrizione)
    .bind(&ricambio.marca)
    .bind(&ricambio.modello_auto)
    .bind(ricambio.anno_da)
    .bind(ricambio.anno_a)
    .bind(&ricambio.categoria)
    .bind(ricambio.fornitore_id)
    .bind(ricambio.giacenza)
    .bind(ricambio.giacenza_minima)
    .bind(ricambio.prezzo_acquisto)
    .bind(ricambio.prezzo_vendita)
    .bind(ricambio.iva_percentuale)
    .bind(&ricambio.posizione)
    .bind(&ricambio.note)
    .bind(id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("ricambio id={id} non trovato")));
    }
    get_ricambio(id, state).await
}

#[tauri::command]
pub async fn delete_ricambio(id: i64, state: State<'_, AppState>) -> Result<(), AppError> {
    let rows = sqlx::query("DELETE FROM ricambi WHERE id=?")
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(AppError::NotFound(format!("ricambio id={id} non trovato")));
    }
    Ok(())
}

#[tauri::command]
pub async fn carico_rapido_ricambio(
    codice: String,
    quantita: i32,
    state: State<'_, AppState>,
) -> Result<RicambioResult, AppError> {
    let codice_lower = codice.trim().to_lowercase();
    if codice_lower.is_empty() {
        return Ok(RicambioResult { trovato: false, ricambio: None, nuova_giacenza: None });
    }

    let ricambio = sqlx::query_as::<_, Ricambio>(
        "SELECT * FROM ricambi WHERE LOWER(codice_interno) = ? OR LOWER(codice_oem) = ? LIMIT 1"
    )
    .bind(&codice_lower)
    .bind(&codice_lower)
    .fetch_optional(&state.db)
    .await?;

    match ricambio {
        None => Ok(RicambioResult { trovato: false, ricambio: None, nuova_giacenza: None }),
        Some(r) => {
            let delta = quantita as i64;
            sqlx::query(
                "UPDATE ricambi SET giacenza = giacenza + ?, updated_at = datetime('now') WHERE id = ?"
            )
            .bind(delta)
            .bind(r.id)
            .execute(&state.db)
            .await?;

            sqlx::query(
                "INSERT INTO movimenti_magazzino (ricambio_id, tipo_movimento, quantita, documento_id, note)
                 VALUES (?, 'carico', ?, NULL, 'carico rapido barcode')"
            )
            .bind(r.id)
            .bind(quantita as f64)
            .execute(&state.db)
            .await?;

            let updated = sqlx::query_as::<_, Ricambio>("SELECT * FROM ricambi WHERE id = ?")
                .bind(r.id)
                .fetch_one(&state.db)
                .await?;

            let nuova_giacenza = updated.giacenza;
            Ok(RicambioResult { trovato: true, ricambio: Some(updated), nuova_giacenza: Some(nuova_giacenza) })
        }
    }
}

#[tauri::command]
pub async fn aggiorna_giacenza(
    ricambio_id: i64,
    tipo_movimento: String,
    quantita: f64,
    documento_id: Option<i64>,
    note: Option<String>,
    state: State<'_, AppState>,
) -> Result<Ricambio, AppError> {
    if !["carico", "scarico", "rettifica"].contains(&tipo_movimento.as_str()) {
        return Err(AppError::Validation("tipo_movimento non valido".into()));
    }

    let mut tx = state.db.begin().await?;

    let delta: f64 = match tipo_movimento.as_str() {
        "carico" => quantita,
        "scarico" => -quantita,
        "rettifica" => quantita,
        _ => unreachable!(),
    };

    let rows = if tipo_movimento == "rettifica" {
        sqlx::query(
            "UPDATE ricambi SET giacenza=?, updated_at=datetime('now') WHERE id=?"
        )
        .bind(quantita as i64)
        .bind(ricambio_id)
        .execute(&mut *tx)
        .await?
        .rows_affected()
    } else {
        sqlx::query(
            "UPDATE ricambi SET giacenza=giacenza+?, updated_at=datetime('now') WHERE id=?"
        )
        .bind(delta as i64)
        .bind(ricambio_id)
        .execute(&mut *tx)
        .await?
        .rows_affected()
    };

    if rows == 0 {
        return Err(AppError::NotFound(format!("ricambio id={ricambio_id} non trovato")));
    }

    sqlx::query(
        "INSERT INTO movimenti_magazzino (ricambio_id, tipo_movimento, quantita, documento_id, note)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(ricambio_id)
    .bind(&tipo_movimento)
    .bind(quantita)
    .bind(documento_id)
    .bind(&note)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let ricambio = sqlx::query_as::<_, Ricambio>("SELECT * FROM ricambi WHERE id=?")
        .bind(ricambio_id)
        .fetch_one(&state.db)
        .await?;
    Ok(ricambio)
}
