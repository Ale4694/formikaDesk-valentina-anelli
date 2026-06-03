use crate::{AppError, AppState};
use crate::models::DashboardStats;
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
