use crate::{AppError, AppState};
use crate::models::{NuovoScontrino, Ricambio, RigaScontrino, Scontrino, ScontrinoCompleto};
use chrono::Local;
use tauri::State;

async fn fetch_scontrino_completo(
    id: i64,
    db: &sqlx::SqlitePool,
) -> Result<ScontrinoCompleto, AppError> {
    let scontrino = sqlx::query_as::<_, Scontrino>(
        "SELECT * FROM scontrini WHERE id=?",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("scontrino id={id} non trovato")))?;

    let righe = sqlx::query_as::<_, RigaScontrino>(
        "SELECT * FROM righe_scontrino WHERE scontrino_id=? ORDER BY id ASC",
    )
    .bind(id)
    .fetch_all(db)
    .await?;

    Ok(ScontrinoCompleto { scontrino, righe })
}

#[tauri::command]
pub async fn get_all_scontrini(
    state: State<'_, AppState>,
) -> Result<Vec<ScontrinoCompleto>, AppError> {
    let scontrini = sqlx::query_as::<_, Scontrino>(
        "SELECT * FROM scontrini ORDER BY data DESC, id DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut result = Vec::with_capacity(scontrini.len());
    for s in scontrini {
        let righe = sqlx::query_as::<_, RigaScontrino>(
            "SELECT * FROM righe_scontrino WHERE scontrino_id=? ORDER BY id ASC",
        )
        .bind(s.id)
        .fetch_all(&state.db)
        .await?;
        result.push(ScontrinoCompleto { scontrino: s, righe });
    }
    Ok(result)
}

#[tauri::command]
pub async fn get_scontrino(
    id: i64,
    state: State<'_, AppState>,
) -> Result<ScontrinoCompleto, AppError> {
    fetch_scontrino_completo(id, &state.db).await
}

#[tauri::command]
pub async fn cerca_ricambio_barcode(
    codice: String,
    state: State<'_, AppState>,
) -> Result<Option<Ricambio>, AppError> {
    let codice = codice.trim().to_string();
    if codice.is_empty() {
        return Ok(None);
    }

    // Exact match first (scanner output)
    let exact = sqlx::query_as::<_, Ricambio>(
        "SELECT * FROM ricambi WHERE codice_interno = ? OR codice_oem = ? LIMIT 1",
    )
    .bind(&codice)
    .bind(&codice)
    .fetch_optional(&state.db)
    .await?;

    if exact.is_some() {
        return Ok(exact);
    }

    // Prefix LIKE fallback (keyboard typing)
    let pattern = format!("{}%", codice);
    let like_result = sqlx::query_as::<_, Ricambio>(
        "SELECT * FROM ricambi WHERE codice_interno LIKE ? OR codice_oem LIKE ? LIMIT 1",
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_optional(&state.db)
    .await?;

    Ok(like_result)
}

#[tauri::command]
pub async fn crea_scontrino(
    scontrino: NuovoScontrino,
    state: State<'_, AppState>,
) -> Result<ScontrinoCompleto, AppError> {
    if scontrino.righe.is_empty() {
        return Err(AppError::Validation("il carrello è vuoto".into()));
    }

    let metodi_validi = ["contanti", "carta", "satispay", "bonifico"];
    if !metodi_validi.contains(&scontrino.metodo_pagamento.as_str()) {
        return Err(AppError::Validation(
            format!("metodo_pagamento '{}' non valido", scontrino.metodo_pagamento),
        ));
    }

    // Calcolo totali
    let mut subtotale = 0.0f64;
    let mut righe_calcolate: Vec<(f64, f64)> = Vec::new(); // (imponibile_riga, totale_riga)
    for riga in &scontrino.righe {
        let imponibile = riga.quantita * riga.prezzo_unitario * (1.0 - riga.sconto_percentuale / 100.0);
        let iva = imponibile * riga.iva_percentuale / 100.0;
        let totale_riga = imponibile + iva;
        subtotale += totale_riga;
        righe_calcolate.push((imponibile, totale_riga));
    }
    let sconto_euro = subtotale * scontrino.sconto_totale / 100.0;
    let totale = (subtotale - sconto_euro).max(0.0);
    let data_oggi = Local::now().format("%Y-%m-%d").to_string();
    let anno = Local::now().format("%Y").to_string();

    let mut tx = state.db.begin().await?;

    // Numero progressivo annuale
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scontrini WHERE strftime('%Y', created_at) = ?",
    )
    .bind(&anno)
    .fetch_one(&mut *tx)
    .await?;
    let numero = format!("SC-{}-{:03}", anno, count + 1);

    let scontrino_id = sqlx::query(
        "INSERT INTO scontrini (numero, data, operatore, totale, sconto_totale, metodo_pagamento, stato, note)
         VALUES (?, ?, ?, ?, ?, ?, 'chiuso', ?)",
    )
    .bind(&numero)
    .bind(&data_oggi)
    .bind(&scontrino.operatore)
    .bind(totale)
    .bind(sconto_euro)
    .bind(&scontrino.metodo_pagamento)
    .bind(&scontrino.note)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    for (riga, (_imponibile, totale_riga)) in scontrino.righe.iter().zip(righe_calcolate.iter()) {
        sqlx::query(
            "INSERT INTO righe_scontrino
             (scontrino_id, ricambio_id, codice, descrizione, quantita, prezzo_unitario,
              sconto_percentuale, totale_riga, iva_percentuale)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(scontrino_id)
        .bind(riga.ricambio_id)
        .bind(&riga.codice)
        .bind(&riga.descrizione)
        .bind(riga.quantita)
        .bind(riga.prezzo_unitario)
        .bind(riga.sconto_percentuale)
        .bind(totale_riga)
        .bind(riga.iva_percentuale)
        .execute(&mut *tx)
        .await?;

        // Scarico giacenza
        if let Some(rid) = riga.ricambio_id {
            sqlx::query(
                "UPDATE ricambi SET giacenza=giacenza-?, updated_at=datetime('now') WHERE id=?",
            )
            .bind(riga.quantita as i64)
            .bind(rid)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                "INSERT INTO movimenti_magazzino (ricambio_id, tipo_movimento, quantita)
                 VALUES (?, 'scarico', ?)",
            )
            .bind(rid)
            .bind(riga.quantita)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    fetch_scontrino_completo(scontrino_id, &state.db).await
}

#[tauri::command]
pub async fn annulla_scontrino(
    id: i64,
    state: State<'_, AppState>,
) -> Result<ScontrinoCompleto, AppError> {
    let s = sqlx::query_as::<_, Scontrino>(
        "SELECT * FROM scontrini WHERE id=?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("scontrino id={id} non trovato")))?;

    if s.stato == "annullato" {
        return Err(AppError::Validation("scontrino già annullato".into()));
    }

    let righe = sqlx::query_as::<_, RigaScontrino>(
        "SELECT * FROM righe_scontrino WHERE scontrino_id=?",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let mut tx = state.db.begin().await?;

    // Ricarico giacenza
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

    sqlx::query("UPDATE scontrini SET stato='annullato' WHERE id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    fetch_scontrino_completo(id, &state.db).await
}
