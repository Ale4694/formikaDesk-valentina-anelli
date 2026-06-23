use crate::{AppError, AppState};
use crate::models::{FatturaReport, ReportMensile, Ricambio, ScadenzaDocumento};
use tauri::State;

#[tauri::command]
pub async fn get_report_mensile(
    anno: i32,
    mese: i32,
    state: State<'_, AppState>,
) -> Result<ReportMensile, AppError> {
    if !(1..=12).contains(&mese) {
        return Err(AppError::Validation("mese non valido (1-12)".into()));
    }
    let periodo = format!("{:04}-{:02}", anno, mese);

    let totale_entrate: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
         WHERE tipo_documento NOT IN ('nota_credito', 'ddt_fornitore') AND stato != 'annullato'
         AND strftime('%Y-%m', data) = ?",
    )
    .bind(&periodo)
    .fetch_one(&state.db)
    .await?;

    let totale_uscite: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
         WHERE tipo_documento='nota_credito' AND stato != 'annullato'
         AND strftime('%Y-%m', data) = ?",
    )
    .bind(&periodo)
    .fetch_one(&state.db)
    .await?;

    let lista_fatture = sqlx::query_as::<_, FatturaReport>(
        "SELECT d.id, d.numero,
                c.ragione_sociale AS cliente_ragione_sociale,
                d.totale_documento, d.stato
         FROM documenti d
         LEFT JOIN clienti c ON d.cliente_id = c.id
         WHERE d.tipo_documento NOT IN ('nota_credito', 'ddt_fornitore')
           AND strftime('%Y-%m', d.data) = ?
         ORDER BY d.data ASC",
    )
    .bind(&periodo)
    .fetch_all(&state.db)
    .await?;

    let ricambi_sotto_scorta = sqlx::query_as::<_, Ricambio>(
        "SELECT * FROM ricambi WHERE giacenza < giacenza_minima ORDER BY descrizione ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let ordini_in_attesa: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ordini_fornitore WHERE stato IN ('bozza','inviato')",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(ReportMensile {
        anno,
        mese,
        totale_entrate,
        totale_uscite,
        saldo: totale_entrate - totale_uscite,
        lista_fatture,
        ricambi_sotto_scorta,
        ordini_in_attesa,
    })
}

#[tauri::command]
pub async fn get_report_giornaliero(
    giorno: String,
    state: State<'_, AppState>,
) -> Result<ReportMensile, AppError> {
    let totale_entrate: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
         WHERE tipo_documento NOT IN ('nota_credito', 'ddt_fornitore') AND stato != 'annullato'
         AND data = ?",
    )
    .bind(&giorno)
    .fetch_one(&state.db)
    .await?;

    let totale_uscite: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
         WHERE tipo_documento='nota_credito' AND stato != 'annullato'
         AND data = ?",
    )
    .bind(&giorno)
    .fetch_one(&state.db)
    .await?;

    let lista_fatture = sqlx::query_as::<_, FatturaReport>(
        "SELECT d.id, d.numero,
                c.ragione_sociale AS cliente_ragione_sociale,
                d.totale_documento, d.stato
         FROM documenti d
         LEFT JOIN clienti c ON d.cliente_id = c.id
         WHERE d.tipo_documento NOT IN ('nota_credito', 'ddt_fornitore')
           AND d.data = ?
         ORDER BY d.data ASC",
    )
    .bind(&giorno)
    .fetch_all(&state.db)
    .await?;

    let ricambi_sotto_scorta = sqlx::query_as::<_, Ricambio>(
        "SELECT * FROM ricambi WHERE giacenza < giacenza_minima ORDER BY descrizione ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let ordini_in_attesa: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ordini_fornitore WHERE stato IN ('bozza','inviato')",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(ReportMensile {
        anno: 0,
        mese: 0,
        totale_entrate,
        totale_uscite,
        saldo: totale_entrate - totale_uscite,
        lista_fatture,
        ricambi_sotto_scorta,
        ordini_in_attesa,
    })
}

#[tauri::command]
pub async fn get_report_annuale(
    anno: i32,
    state: State<'_, AppState>,
) -> Result<ReportMensile, AppError> {
    let dal = format!("{}-01-01", anno);
    let al = format!("{}-12-31", anno);

    let totale_entrate: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
         WHERE tipo_documento NOT IN ('nota_credito', 'ddt_fornitore') AND stato != 'annullato'
         AND data >= ? AND data <= ?",
    )
    .bind(&dal)
    .bind(&al)
    .fetch_one(&state.db)
    .await?;

    let totale_uscite: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
         WHERE tipo_documento='nota_credito' AND stato != 'annullato'
         AND data >= ? AND data <= ?",
    )
    .bind(&dal)
    .bind(&al)
    .fetch_one(&state.db)
    .await?;

    let lista_fatture = sqlx::query_as::<_, FatturaReport>(
        "SELECT d.id, d.numero,
                c.ragione_sociale AS cliente_ragione_sociale,
                d.totale_documento, d.stato
         FROM documenti d
         LEFT JOIN clienti c ON d.cliente_id = c.id
         WHERE d.tipo_documento NOT IN ('nota_credito', 'ddt_fornitore')
           AND d.data >= ? AND d.data <= ?
         ORDER BY d.data ASC",
    )
    .bind(&dal)
    .bind(&al)
    .fetch_all(&state.db)
    .await?;

    let ricambi_sotto_scorta = sqlx::query_as::<_, Ricambio>(
        "SELECT * FROM ricambi WHERE giacenza < giacenza_minima ORDER BY descrizione ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let ordini_in_attesa: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ordini_fornitore WHERE stato IN ('bozza','inviato')",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(ReportMensile {
        anno,
        mese: 0,
        totale_entrate,
        totale_uscite,
        saldo: totale_entrate - totale_uscite,
        lista_fatture,
        ricambi_sotto_scorta,
        ordini_in_attesa,
    })
}

#[tauri::command]
pub async fn get_report_personalizzato(
    dal: String,
    al: String,
    state: State<'_, AppState>,
) -> Result<ReportMensile, AppError> {
    let totale_entrate: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
         WHERE tipo_documento NOT IN ('nota_credito', 'ddt_fornitore') AND stato != 'annullato'
         AND data >= ? AND data <= ?",
    )
    .bind(&dal)
    .bind(&al)
    .fetch_one(&state.db)
    .await?;

    let totale_uscite: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
         WHERE tipo_documento='nota_credito' AND stato != 'annullato'
         AND data >= ? AND data <= ?",
    )
    .bind(&dal)
    .bind(&al)
    .fetch_one(&state.db)
    .await?;

    let lista_fatture = sqlx::query_as::<_, FatturaReport>(
        "SELECT d.id, d.numero,
                c.ragione_sociale AS cliente_ragione_sociale,
                d.totale_documento, d.stato
         FROM documenti d
         LEFT JOIN clienti c ON d.cliente_id = c.id
         WHERE d.tipo_documento NOT IN ('nota_credito', 'ddt_fornitore')
           AND d.data >= ? AND d.data <= ?
         ORDER BY d.data ASC",
    )
    .bind(&dal)
    .bind(&al)
    .fetch_all(&state.db)
    .await?;

    let ricambi_sotto_scorta = sqlx::query_as::<_, Ricambio>(
        "SELECT * FROM ricambi WHERE giacenza < giacenza_minima ORDER BY descrizione ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let ordini_in_attesa: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ordini_fornitore WHERE stato IN ('bozza','inviato')",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(ReportMensile {
        anno: 0,
        mese: 0,
        totale_entrate,
        totale_uscite,
        saldo: totale_entrate - totale_uscite,
        lista_fatture,
        ricambi_sotto_scorta,
        ordini_in_attesa,
    })
}

#[tauri::command]
pub async fn get_scadenzario(
    state: State<'_, AppState>,
) -> Result<Vec<ScadenzaDocumento>, AppError> {
    let scadenze = sqlx::query_as::<_, ScadenzaDocumento>(
        "SELECT d.id, d.numero, d.tipo_documento, d.data,
                d.scadenza_pagamento, d.totale_documento, d.stato,
                d.cliente_id,
                c.ragione_sociale AS cliente_ragione_sociale,
                (julianday(d.scadenza_pagamento) - julianday('now')) AS giorni_alla_scadenza
         FROM documenti d
         LEFT JOIN clienti c ON d.cliente_id = c.id
         WHERE d.tipo_documento = 'fattura'
           AND d.stato NOT IN ('pagato','annullato')
           AND d.scadenza_pagamento IS NOT NULL
         ORDER BY d.scadenza_pagamento ASC",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(scadenze)
}
