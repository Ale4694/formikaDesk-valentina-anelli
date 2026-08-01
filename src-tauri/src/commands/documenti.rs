use crate::{AppError, AppState};
use crate::models::{ArticoloStorico, Documento, DocumentoCompleto, NuovoDocumento, PaginatedResult, RigaDocumento};
use chrono::NaiveDate;
use tauri::State;

/// Causale di default storica: Buono/Preventivo stampavano "CONSEGNA", gli altri tipi "Vendita".
fn causale_trasporto_default(tipo: &str) -> &'static str {
    if tipo == "buono" || tipo == "preventivo" { "CONSEGNA" } else { "Vendita" }
}

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
pub async fn get_documenti_paginated(
    page: i64,
    page_size: i64,
    stato: String,
    tipo: String,
    search: String,
    sort_col: String,
    sort_dir: String,
    state: State<'_, AppState>,
) -> Result<PaginatedResult<Documento>, AppError> {
    let page_size = page_size.clamp(1, 200);
    let offset = page.max(0) * page_size;
    let allowed = ["data","numero","tipo_documento","stato","totale_documento","cliente_id","created_at"];
    let col = if allowed.contains(&sort_col.as_str()) { sort_col.as_str() } else { "data" };
    let dir = if sort_dir == "desc" { "DESC" } else { "ASC" };
    let stato_p: Option<&str> = if stato == "tutti" || stato.is_empty() { None } else { Some(&stato) };
    let tipo_p: Option<&str>  = if tipo  == "tutti" || tipo.is_empty()  { None } else { Some(&tipo)  };
    let search_like: Option<String> = if search.trim().is_empty() { None } else { Some(format!("%{}%", search.trim())) };
    let search_p: Option<&str> = search_like.as_deref();

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM documenti d \
         LEFT JOIN clienti c ON c.id = d.cliente_id \
         WHERE (? IS NULL OR d.stato = ?) \
           AND (? IS NULL OR d.tipo_documento = ?) \
           AND (? IS NULL OR d.numero LIKE ? OR c.ragione_sociale LIKE ?)"
    )
    .bind(stato_p).bind(stato_p)
    .bind(tipo_p).bind(tipo_p)
    .bind(search_p).bind(search_p).bind(search_p)
    .fetch_one(&state.db).await?;

    let sql = format!(
        "SELECT d.* FROM documenti d \
         LEFT JOIN clienti c ON c.id = d.cliente_id \
         WHERE (? IS NULL OR d.stato = ?) \
           AND (? IS NULL OR d.tipo_documento = ?) \
           AND (? IS NULL OR d.numero LIKE ? OR c.ragione_sociale LIKE ?) \
         ORDER BY d.{col} {dir}, d.id DESC LIMIT ? OFFSET ?"
    );
    let items = sqlx::query_as::<_, Documento>(&sql)
        .bind(stato_p).bind(stato_p)
        .bind(tipo_p).bind(tipo_p)
        .bind(search_p).bind(search_p).bind(search_p)
        .bind(page_size).bind(offset)
        .fetch_all(&state.db).await?;

    Ok(PaginatedResult { items, total, page, page_size })
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

    let tipi_senza_scadenza = ["vendita_banco", "buono"];
    let scadenza_pagamento = if tipi_senza_scadenza.contains(&doc.tipo_documento.as_str()) {
        None
    } else {
        let giorni = doc.giorni_pagamento.unwrap_or(30);
        NaiveDate::parse_from_str(&doc.data, "%Y-%m-%d")
            .ok()
            .map(|d| (d + chrono::Duration::days(giorni)).to_string())
    };
    let giorni = doc.giorni_pagamento.unwrap_or(30);

    let is_fattura_differita: i64 = if doc.tipo_documento == "fattura_differita" { 1 } else { 0 };
    let ddt_collegati_json: Option<String> = doc
        .ddt_collegati
        .as_ref()
        .map(|ids| serde_json::to_string(ids).unwrap_or_default());

    let causale_trasporto = doc.causale_trasporto.clone()
        .unwrap_or_else(|| causale_trasporto_default(&doc.tipo_documento).to_string());
    let trasporto_a_cura = doc.trasporto_a_cura.clone().unwrap_or_else(|| "Destinatario".to_string());
    let n_colli = doc.n_colli.unwrap_or(0);
    let spese_varie = doc.spese_varie.unwrap_or(0.0);
    let spese_incasso = doc.spese_incasso.unwrap_or(0.0);
    let mostra_iban = doc.mostra_iban.unwrap_or(true);

    let mut tx = state.db.begin().await?;

    let doc_id = sqlx::query(
        "INSERT INTO documenti (tipo_documento, numero, data, cliente_id, fornitore_id, note,
         totale_imponibile, totale_iva, totale_documento, scadenza_pagamento, giorni_pagamento,
         is_fattura_differita, ddt_collegati, stato,
         vettore, data_ora_ritiro, n_colli, aspetto_esteriore_beni, porto,
         causale_trasporto, trasporto_a_cura, banca_appoggio, agente, bolli_art15,
         spese_varie, spese_incasso, mostra_iban)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'confermato',
         ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
    .bind(is_fattura_differita)
    .bind(&ddt_collegati_json)
    .bind(&doc.vettore)
    .bind(&doc.data_ora_ritiro)
    .bind(n_colli)
    .bind(&doc.aspetto_esteriore_beni)
    .bind(&doc.porto)
    .bind(&causale_trasporto)
    .bind(&trasporto_a_cura)
    .bind(&doc.banca_appoggio)
    .bind(&doc.agente)
    .bind(&doc.bolli_art15)
    .bind(spese_varie)
    .bind(spese_incasso)
    .bind(mostra_iban)
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
            let tipi_scarico = ["fattura", "ddt", "vendita_banco", "buono", "fattura_differita"];
            if tipi_scarico.contains(&doc.tipo_documento.as_str()) {
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

    // Per fattura_differita: segna i DDT collegati come fatturati
    if doc.tipo_documento == "fattura_differita" {
        if let Some(ids) = &doc.ddt_collegati {
            for ddt_id in ids {
                sqlx::query(
                    "UPDATE documenti SET fatturato=1, updated_at=datetime('now') WHERE id=?",
                )
                .bind(ddt_id)
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

#[tauri::command]
pub async fn update_documento(
    id: i64,
    doc: NuovoDocumento,
    state: State<'_, AppState>,
) -> Result<DocumentoCompleto, AppError> {
    if doc.numero.trim().is_empty() {
        return Err(AppError::Validation("numero documento obbligatorio".into()));
    }
    if doc.righe.is_empty() {
        return Err(AppError::Validation("il documento deve avere almeno una riga".into()));
    }

    let mut tx = state.db.begin().await?;

    let old = sqlx::query_as::<_, Documento>("SELECT * FROM documenti WHERE id=?")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("documento id={id} non trovato")))?;

    let tipi_scarico = ["fattura", "ddt", "vendita_banco", "buono", "fattura_differita"];
    let tipi_carico  = ["ddt_fornitore"];

    // Ripristina magazzino dalle righe vecchie
    if tipi_scarico.contains(&old.tipo_documento.as_str())
        || tipi_carico.contains(&old.tipo_documento.as_str())
    {
        let old_righe = sqlx::query_as::<_, RigaDocumento>(
            "SELECT * FROM righe_documento WHERE documento_id=?",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

        for riga in &old_righe {
            if let Some(rid) = riga.ricambio_id {
                if tipi_scarico.contains(&old.tipo_documento.as_str()) {
                    // Ripristina scarico: giacenza++
                    sqlx::query(
                        "UPDATE ricambi SET giacenza=giacenza+?, updated_at=datetime('now') WHERE id=?",
                    )
                    .bind(riga.quantita as i64)
                    .bind(rid)
                    .execute(&mut *tx)
                    .await?;
                } else {
                    // Ripristina carico: giacenza--
                    sqlx::query(
                        "UPDATE ricambi SET giacenza=giacenza-?, updated_at=datetime('now') WHERE id=?",
                    )
                    .bind(riga.quantita as i64)
                    .bind(rid)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }
        sqlx::query("DELETE FROM movimenti_magazzino WHERE documento_id=?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    // Decollegare i vecchi DDT se era fattura_differita
    if old.tipo_documento == "fattura_differita" {
        if let Some(ids_json) = &old.ddt_collegati {
            if let Ok(ids) = serde_json::from_str::<Vec<i64>>(ids_json) {
                for ddt_id in ids {
                    sqlx::query(
                        "UPDATE documenti SET fatturato=0, updated_at=datetime('now') WHERE id=?",
                    )
                    .bind(ddt_id)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }
    }

    // Calcola nuovi totali
    let mut totale_imponibile = 0.0f64;
    let mut totale_iva = 0.0f64;
    for riga in &doc.righe {
        let lordo = riga.quantita * riga.prezzo_unitario;
        let sconto = lordo * (riga.sconto_percentuale / 100.0);
        let imponibile = lordo - sconto;
        totale_imponibile += imponibile;
        totale_iva += imponibile * (riga.iva_percentuale / 100.0);
    }
    let totale_documento = totale_imponibile + totale_iva;

    let tipi_senza_scadenza = ["vendita_banco", "buono"];
    let scadenza_pagamento = if tipi_senza_scadenza.contains(&doc.tipo_documento.as_str()) {
        None
    } else {
        let giorni = doc.giorni_pagamento.unwrap_or(30);
        NaiveDate::parse_from_str(&doc.data, "%Y-%m-%d")
            .ok()
            .map(|d| (d + chrono::Duration::days(giorni)).to_string())
    };
    let giorni = doc.giorni_pagamento.unwrap_or(30);
    let is_fattura_differita: i64 = if doc.tipo_documento == "fattura_differita" { 1 } else { 0 };
    let ddt_collegati_json: Option<String> = doc
        .ddt_collegati
        .as_ref()
        .map(|ids| serde_json::to_string(ids).unwrap_or_default());

    let causale_trasporto = doc.causale_trasporto.clone()
        .unwrap_or_else(|| causale_trasporto_default(&doc.tipo_documento).to_string());
    let trasporto_a_cura = doc.trasporto_a_cura.clone().unwrap_or_else(|| "Destinatario".to_string());
    let n_colli = doc.n_colli.unwrap_or(0);
    let spese_varie = doc.spese_varie.unwrap_or(0.0);
    let spese_incasso = doc.spese_incasso.unwrap_or(0.0);
    let mostra_iban = doc.mostra_iban.unwrap_or(true);

    sqlx::query(
        "UPDATE documenti SET tipo_documento=?, numero=?, data=?, cliente_id=?, fornitore_id=?,
         note=?, totale_imponibile=?, totale_iva=?, totale_documento=?,
         scadenza_pagamento=?, giorni_pagamento=?, is_fattura_differita=?, ddt_collegati=?,
         vettore=?, data_ora_ritiro=?, n_colli=?, aspetto_esteriore_beni=?, porto=?,
         causale_trasporto=?, trasporto_a_cura=?, banca_appoggio=?, agente=?, bolli_art15=?,
         spese_varie=?, spese_incasso=?, mostra_iban=?,
         updated_at=datetime('now') WHERE id=?",
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
    .bind(is_fattura_differita)
    .bind(&ddt_collegati_json)
    .bind(&doc.vettore)
    .bind(&doc.data_ora_ritiro)
    .bind(n_colli)
    .bind(&doc.aspetto_esteriore_beni)
    .bind(&doc.porto)
    .bind(&causale_trasporto)
    .bind(&trasporto_a_cura)
    .bind(&doc.banca_appoggio)
    .bind(&doc.agente)
    .bind(&doc.bolli_art15)
    .bind(spese_varie)
    .bind(spese_incasso)
    .bind(mostra_iban)
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM righe_documento WHERE documento_id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    for riga in &doc.righe {
        let lordo = riga.quantita * riga.prezzo_unitario;
        let sconto = lordo * (riga.sconto_percentuale / 100.0);
        let imponibile = lordo - sconto;
        let iva_riga = imponibile * (riga.iva_percentuale / 100.0);
        let totale_riga = imponibile + iva_riga;

        sqlx::query(
            "INSERT INTO righe_documento (documento_id, ricambio_id, descrizione, quantita,
             prezzo_unitario, sconto_percentuale, iva_percentuale, imponibile, totale_iva,
             totale_riga, ordine) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(id)
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
            if tipi_scarico.contains(&doc.tipo_documento.as_str()) {
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
                .bind(id)
                .execute(&mut *tx)
                .await?;
            } else if doc.tipo_documento == "ddt_fornitore" {
                sqlx::query(
                    "UPDATE ricambi SET giacenza=giacenza+?, updated_at=datetime('now') WHERE id=?",
                )
                .bind(riga.quantita as i64)
                .bind(rid)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO movimenti_magazzino (ricambio_id, tipo_movimento, quantita, documento_id, note)
                     VALUES (?, 'carico', ?, ?, 'DDT Fornitore')",
                )
                .bind(rid)
                .bind(riga.quantita)
                .bind(id)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    // Collega nuovi DDT se fattura_differita
    if doc.tipo_documento == "fattura_differita" {
        if let Some(ids) = &doc.ddt_collegati {
            for ddt_id in ids {
                sqlx::query(
                    "UPDATE documenti SET fatturato=1, updated_at=datetime('now') WHERE id=?",
                )
                .bind(ddt_id)
                .execute(&mut *tx)
                .await?;
            }
        }
    }

    tx.commit().await?;
    get_documento(id, state).await
}

#[tauri::command]
pub async fn delete_documento(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut tx = state.db.begin().await?;

    let doc = sqlx::query_as::<_, Documento>("SELECT * FROM documenti WHERE id=?")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("documento id={id} non trovato")))?;

    let tipi_scarico = ["fattura", "ddt", "vendita_banco", "buono", "fattura_differita"];
    let tipi_carico  = ["ddt_fornitore"];

    // Ripristina magazzino
    if tipi_scarico.contains(&doc.tipo_documento.as_str())
        || tipi_carico.contains(&doc.tipo_documento.as_str())
    {
        let righe = sqlx::query_as::<_, RigaDocumento>(
            "SELECT * FROM righe_documento WHERE documento_id=?",
        )
        .bind(id)
        .fetch_all(&mut *tx)
        .await?;

        for riga in &righe {
            if let Some(rid) = riga.ricambio_id {
                if tipi_scarico.contains(&doc.tipo_documento.as_str()) {
                    sqlx::query(
                        "UPDATE ricambi SET giacenza=giacenza+?, updated_at=datetime('now') WHERE id=?",
                    )
                    .bind(riga.quantita as i64)
                    .bind(rid)
                    .execute(&mut *tx)
                    .await?;
                } else {
                    // ddt_fornitore: ripristina il carico (giacenza--)
                    sqlx::query(
                        "UPDATE ricambi SET giacenza=giacenza-?, updated_at=datetime('now') WHERE id=?",
                    )
                    .bind(riga.quantita as i64)
                    .bind(rid)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }
    }

    // Decollegare DDT se era fattura_differita
    if doc.tipo_documento == "fattura_differita" {
        if let Some(ids_json) = &doc.ddt_collegati {
            if let Ok(ids) = serde_json::from_str::<Vec<i64>>(ids_json) {
                for ddt_id in ids {
                    sqlx::query(
                        "UPDATE documenti SET fatturato=0, updated_at=datetime('now') WHERE id=?",
                    )
                    .bind(ddt_id)
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }
    }

    sqlx::query("DELETE FROM movimenti_magazzino WHERE documento_id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM righe_documento WHERE documento_id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM documenti WHERE id=?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

// Nota: la fonte dati e' v_storico_vendite (vedi migration
// 20260801_prezzi_cliente_storico.sql), che filtra per tipo
// documento e stato. Prima usava righe_documento/documenti senza
// filtri: poteva suggerire il prezzo di un preventivo mai concluso
// o di un documento annullato. Firma e forma del risultato restano
// identiche: questo comando alimenta il picker storico (stella) in
// NuovaFattura.svelte e quel comportamento UI non cambia.
#[tauri::command]
pub async fn get_storico_articoli_cliente(
    cliente_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ArticoloStorico>, AppError> {
    let rows = sqlx::query_as::<_, ArticoloStorico>(
        "SELECT v.articolo_id as ricambio_id,
                v.codice_articolo as codice_interno,
                v.descrizione,
                COALESCE((
                    SELECT v2.prezzo_unitario FROM v_storico_vendite v2
                    WHERE v2.articolo_id = v.articolo_id
                      AND v2.cliente_id = ?
                    ORDER BY v2.data DESC, v2.documento_id DESC LIMIT 1
                ), 0.0) as prezzo_unitario,
                CAST(SUM(v.quantita) AS REAL) as quantita_totale,
                COUNT(*) as frequenza
         FROM v_storico_vendite v
         WHERE v.cliente_id = ?
         GROUP BY v.articolo_id, v.codice_articolo, v.descrizione
         ORDER BY frequenza DESC, MAX(v.data) DESC
         LIMIT 20",
    )
    .bind(cliente_id)
    .bind(cliente_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}
