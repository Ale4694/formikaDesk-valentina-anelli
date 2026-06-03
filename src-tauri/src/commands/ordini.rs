use crate::{AppError, AppState};
use crate::models::{NuovoOrdineFornitore, OrdineCompleto, OrdineFornitore, RigaOrdine};
use tauri::State;

async fn fetch_ordine_completo(id: i64, db: &sqlx::SqlitePool) -> Result<OrdineCompleto, AppError> {
    let ordine = sqlx::query_as::<_, OrdineFornitore>(
        "SELECT * FROM ordini_fornitore WHERE id=?",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("ordine id={id} non trovato")))?;

    let righe = sqlx::query_as::<_, RigaOrdine>(
        "SELECT * FROM righe_ordine WHERE ordine_id=? ORDER BY id ASC",
    )
    .bind(id)
    .fetch_all(db)
    .await?;

    Ok(OrdineCompleto { ordine, righe })
}

#[tauri::command]
pub async fn get_all_ordini(state: State<'_, AppState>) -> Result<Vec<OrdineCompleto>, AppError> {
    let ordini = sqlx::query_as::<_, OrdineFornitore>(
        "SELECT * FROM ordini_fornitore ORDER BY data DESC, id DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut result = Vec::with_capacity(ordini.len());
    for o in ordini {
        let righe = sqlx::query_as::<_, RigaOrdine>(
            "SELECT * FROM righe_ordine WHERE ordine_id=? ORDER BY id ASC",
        )
        .bind(o.id)
        .fetch_all(&state.db)
        .await?;
        result.push(OrdineCompleto { ordine: o, righe });
    }
    Ok(result)
}

#[tauri::command]
pub async fn create_ordine_fornitore(
    ordine: NuovoOrdineFornitore,
    state: State<'_, AppState>,
) -> Result<OrdineCompleto, AppError> {
    if ordine.righe.is_empty() {
        return Err(AppError::Validation(
            "l'ordine deve avere almeno una riga".into(),
        ));
    }

    let stato = ordine.stato.as_deref().unwrap_or("bozza");
    let stati_validi = ["bozza", "inviato", "ricevuto", "annullato"];
    if !stati_validi.contains(&stato) {
        return Err(AppError::Validation(format!("stato '{stato}' non valido")));
    }

    let totale: f64 = ordine
        .righe
        .iter()
        .map(|r| r.quantita * r.prezzo_unitario)
        .sum();

    let mut tx = state.db.begin().await?;

    let ordine_id = sqlx::query(
        "INSERT INTO ordini_fornitore (fornitore_id, data, stato, note, totale)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(ordine.fornitore_id)
    .bind(&ordine.data)
    .bind(stato)
    .bind(&ordine.note)
    .bind(totale)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    for riga in &ordine.righe {
        let totale_riga = riga.quantita * riga.prezzo_unitario;
        sqlx::query(
            "INSERT INTO righe_ordine (ordine_id, ricambio_id, descrizione, quantita, prezzo_unitario, totale_riga)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(ordine_id)
        .bind(riga.ricambio_id)
        .bind(&riga.descrizione)
        .bind(riga.quantita)
        .bind(riga.prezzo_unitario)
        .bind(totale_riga)
        .execute(&mut *tx)
        .await?;

        if stato == "ricevuto" {
            if let Some(rid) = riga.ricambio_id {
                sqlx::query(
                    "UPDATE ricambi SET giacenza=giacenza+?, updated_at=datetime('now') WHERE id=?",
                )
                .bind(riga.quantita as i64)
                .bind(rid)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO movimenti_magazzino (ricambio_id, tipo_movimento, quantita)
                     VALUES (?, 'carico', ?)",
                )
                .bind(rid)
                .bind(riga.quantita)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;
    fetch_ordine_completo(ordine_id, &state.db).await
}

#[tauri::command]
pub async fn update_stato_ordine(
    id: i64,
    stato: String,
    state: State<'_, AppState>,
) -> Result<OrdineCompleto, AppError> {
    let stati_validi = ["bozza", "inviato", "ricevuto", "annullato"];
    if !stati_validi.contains(&stato.as_str()) {
        return Err(AppError::Validation(format!("stato '{stato}' non valido")));
    }

    let ordine_corrente = sqlx::query_as::<_, OrdineFornitore>(
        "SELECT * FROM ordini_fornitore WHERE id=?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("ordine id={id} non trovato")))?;

    if ordine_corrente.stato == stato {
        return fetch_ordine_completo(id, &state.db).await;
    }

    let mut tx = state.db.begin().await?;

    // Carico automatico quando si passa a "ricevuto"
    if stato == "ricevuto" && ordine_corrente.stato != "ricevuto" {
        let righe = sqlx::query_as::<_, RigaOrdine>(
            "SELECT * FROM righe_ordine WHERE ordine_id=?",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

        for riga in &righe {
            if let Some(rid) = riga.ricambio_id {
                sqlx::query(
                    "UPDATE ricambi SET giacenza=giacenza+?, updated_at=datetime('now') WHERE id=?",
                )
                .bind(riga.quantita as i64)
                .bind(rid)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO movimenti_magazzino (ricambio_id, tipo_movimento, quantita)
                     VALUES (?, 'carico', ?)",
                )
                .bind(rid)
                .bind(riga.quantita)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    sqlx::query(
        "UPDATE ordini_fornitore SET stato=?, updated_at=datetime('now') WHERE id=?",
    )
    .bind(&stato)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    fetch_ordine_completo(id, &state.db).await
}
