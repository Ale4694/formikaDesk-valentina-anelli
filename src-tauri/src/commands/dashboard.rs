use crate::{AppError, AppState};
use crate::models::{ClienteAttivo, ClienteTopDocumenti, DashboardStats};
use tauri::State;

#[tauri::command]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<DashboardStats, AppError> {
    let totale_clienti: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM clienti")
            .fetch_one(&state.db).await?;

    let totale_fornitori: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM fornitori")
            .fetch_one(&state.db).await?;

    let totale_ricambi: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ricambi")
            .fetch_one(&state.db).await?;

    let ricambi_sotto_scorta: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM ricambi WHERE giacenza < giacenza_minima")
            .fetch_one(&state.db).await?;

    let fatture_mese_corrente: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM documenti
         WHERE tipo_documento='fattura'
         AND strftime('%Y-%m', data) = strftime('%Y-%m', 'now')"
    ).fetch_one(&state.db).await?;

    let fatturato_mese_corrente: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
         WHERE tipo_documento='fattura' AND stato != 'annullato'
         AND strftime('%Y-%m', data) = strftime('%Y-%m', 'now')"
    ).fetch_one(&state.db).await?;

    let fatture_non_pagate: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM documenti
         WHERE tipo_documento='fattura' AND stato='confermato'"
    ).fetch_one(&state.db).await?;

    let valore_magazzino: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(giacenza * prezzo_acquisto), 0.0) FROM ricambi"
    ).fetch_one(&state.db).await?;

    let fatture_scadute: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM documenti
         WHERE tipo_documento='fattura'
           AND stato NOT IN ('pagato','annullato')
           AND scadenza_pagamento IS NOT NULL
           AND scadenza_pagamento < date('now')"
    ).fetch_one(&state.db).await?;

    let scontrini_oggi: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM scontrini
         WHERE stato = 'chiuso' AND date(data) = date('now')"
    ).fetch_one(&state.db).await?;

    let incasso_oggi: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(totale), 0.0) FROM scontrini
         WHERE stato = 'chiuso' AND date(data) = date('now')"
    ).fetch_one(&state.db).await?;

    Ok(DashboardStats {
        totale_clienti,
        totale_fornitori,
        totale_ricambi,
        ricambi_sotto_scorta,
        fatture_mese_corrente,
        fatturato_mese_corrente,
        fatture_non_pagate,
        valore_magazzino,
        fatture_scadute,
        scontrini_oggi,
        incasso_oggi,
    })
}

/// Top 5 clienti per fatturato dell'anno solare corrente.
/// Filtro IDENTICO a quello di get_riepilogo_cliente (vedi
/// commands/prezzi_cliente.rs): stesso tipo_documento, stesso stato,
/// stesso anno. Il numero deve poter essere verificato dall'utente
/// aprendo la scheda del singolo cliente.
#[tauri::command]
pub async fn get_clienti_piu_attivi(state: State<'_, AppState>) -> Result<Vec<ClienteAttivo>, AppError> {
    let rows = sqlx::query_as::<_, ClienteAttivo>(
        "SELECT
            c.id AS cliente_id,
            c.ragione_sociale AS ragione_sociale,
            COUNT(*) AS numero_documenti,
            SUM(d.totale_documento) AS fatturato
         FROM documenti d
         JOIN clienti c ON c.id = d.cliente_id
         WHERE d.tipo_documento = 'fattura'
           AND d.stato NOT IN ('annullato', 'bozza')
           AND strftime('%Y', d.data) = strftime('%Y', 'now')
         GROUP BY c.id, c.ragione_sociale
         ORDER BY fatturato DESC
         LIMIT 5",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

/// Top 8 clienti per numero di documenti dell'anno corrente (non per
/// fatturato: vedi commento su ClienteTopDocumenti). Alimenta il
/// pannello F2 sul campo cliente di NuovaFattura.svelte: chi torna
/// spesso, per compilare il documento in fretta.
#[tauri::command]
pub async fn get_clienti_top_documenti(state: State<'_, AppState>) -> Result<Vec<ClienteTopDocumenti>, AppError> {
    let rows = sqlx::query_as::<_, ClienteTopDocumenti>(
        "SELECT
            c.id AS cliente_id,
            c.ragione_sociale AS ragione_sociale,
            c.citta AS citta,
            COUNT(*) AS numero_documenti,
            MAX(d.data) AS ultimo_documento
         FROM documenti d
         JOIN clienti c ON c.id = d.cliente_id
         WHERE d.stato NOT IN ('annullato', 'bozza')
           AND strftime('%Y', d.data) = strftime('%Y', 'now')
         GROUP BY c.id, c.ragione_sociale, c.citta
         ORDER BY numero_documenti DESC
         LIMIT 8",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}
