use crate::{AppError, AppState};
use crate::models::{Documento, DocumentoCompleto, NuovoDocumento, RigaDocumento};
use chrono::NaiveDate;
use tauri::State;

#[tauri::command]
pub async fn get_all_documenti(state: State<'_, AppState>) -> Result<Vec<Documento>, AppError> {
    let docs = sqlx::query_as::<_, Documento>(
        "SELECT * FROM documenti ORDER BY data DESC, id DESC",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(docs)
}

#[tauri::command]
pub async fn get_documento(
    id: i64,
    state: State<'_, AppState>,
) -> Result<DocumentoCompleto, AppError> {
    let documento = sqlx::query_as::<_, Documento>("SELECT * FROM documenti WHERE id=?")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("documento id={id} non trovato")))?;

    let righe = sqlx::query_as::<_, RigaDocumento>(
        "SELECT * FROM righe_documento WHERE documento_id=? ORDER BY ordine ASC",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(DocumentoCompleto { documento, righe })
}

#[tauri::command]
pub async fn create_documento(
    doc: NuovoDocumento,
    state: State<'_, AppState>,
) -> Result<DocumentoCompleto, AppError> {
    if doc.numero.trim().is_empty() {
        return Err(AppError::Validation("numero documento obbligatorio".into()));
    }
    if doc.righe.is_empty() {
        return Err(AppError::Validation(
            "il documento deve avere almeno una riga".into(),
        ));
    }

    let mut totale_imponibile = 0.0f64;
    let mut totale_iva = 0.0f64;

    for riga in &doc.righe {
        let lordo = riga.quantita * riga.prezzo_unitario;
        let sconto = lordo * (riga.sconto_percentuale / 100.0);
        let imponibile = lordo - sconto;
        let iva = imponibile * (riga.iva_percentuale / 100.0);
        totale_imponibile += imponibile;
        totale_iva += iva;
    }
    let totale_documento = totale_imponibile + totale_iva;

    let giorni = doc.giorni_pagamento.unwrap_or(30);
    let scadenza_pagamento = NaiveDate::parse_from_str(&doc.data, "%Y-%m-%d")
        .ok()
        .map(|d| (d + chrono::Duration::days(giorni)).to_string());

    let mut tx = state.db.begin().await?;

    let doc_id = sqlx::query(
        "INSERT INTO documenti (tipo_documento, numero, data, cliente_id, fornitore_id, note,
         totale_imponibile, totale_iva, totale_documento, scadenza_pagamento, giorni_pagamento)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&doc.tipo_documento)
    .bind(&doc.numero)
    .bind(&doc.data)
    .bind(doc.cliente_id)
    .bind(doc.fornitore_id)
    .bind(&doc.note)
    .bind(totale_imponibile)
    .bind(totale_iva)
    .bind(totale_documento)
    .bind(&scadenza_pagamento)
    .bind(giorni)
    .execute(&mut *tx)
    .await?
    .last_insert_rowid();

    for riga in &doc.righe {
        let lordo = riga.quantita * riga.prezzo_unitario;
        let sconto = lordo * (riga.sconto_percentuale / 100.0);
        let imponibile = lordo - sconto;
        let iva_riga = imponibile * (riga.iva_percentuale / 100.0);
        let totale_riga = imponibile + iva_riga;

        sqlx::query(
            "INSERT INTO righe_documento (documento_id, ricambio_id, descrizione, quantita,
             prezzo_unitario, sconto_percentuale, iva_percentuale, imponibile, totale_iva, totale_riga, ordine)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(doc_id)
        .bind(riga.ricambio_id)
        .bind(&riga.descrizione)
        .bind(riga.quantita)
        .bind(riga.prezzo_unitario)
        .bind(riga.sconto_percentuale)
        .bind(riga.iva_percentuale)
        .bind(imponibile)
        .bind(iva_riga)
        .bind(totale_riga)
        .bind(riga.ordine)
        .execute(&mut *tx)
        .await?;

        if let Some(rid) = riga.ricambio_id {
            if doc.tipo_documento == "fattura" || doc.tipo_documento == "ddt" {
                sqlx::query(
                    "UPDATE ricambi SET giacenza=giacenza-?, updated_at=datetime('now') WHERE id=?",
                )
                .bind(riga.quantita as i64)
                .bind(rid)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO movimenti_magazzino (ricambio_id, tipo_movimento, quantita, documento_id)
                     VALUES (?, 'scarico', ?, ?)",
                )
                .bind(rid)
                .bind(riga.quantita)
                .bind(doc_id)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;
    get_documento(doc_id, state).await
}

#[tauri::command]
pub async fn update_stato_documento(
    id: i64,
    stato: String,
    data_pagamento: Option<String>,
    metodo_pagamento: Option<String>,
    riferimento_pagamento: Option<String>,
    note_pagamento: Option<String>,
    state: State<'_, AppState>,
) -> Result<Documento, AppError> {
    let stati_validi = ["bozza", "confermato", "pagato", "annullato"];
    if !stati_validi.contains(&stato.as_str()) {
        return Err(AppError::Validation(format!("stato '{}' non valido", stato)));
    }

    let rows = if stato == "pagato" {
        sqlx::query(
            "UPDATE documenti SET stato=?, data_pagamento=?, metodo_pagamento=?, \
             riferimento_pagamento=?, note_pagamento=?, updated_at=datetime('now') WHERE id=?",
        )
        .bind(&stato)
        .bind(&data_pagamento)
        .bind(&metodo_pagamento)
        .bind(&riferimento_pagamento)
        .bind(&note_pagamento)
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected()
    } else {
        sqlx::query(
            "UPDATE documenti SET stato=?, updated_at=datetime('now') WHERE id=?",
        )
        .bind(&stato)
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected()
    };

    if rows == 0 {
        return Err(AppError::NotFound(format!("documento id={id} non trovato")));
    }

    let doc = sqlx::query_as::<_, Documento>("SELECT * FROM documenti WHERE id=?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    Ok(doc)
}
