use crate::models::{
    ArticoloVendutoCliente, DocumentoCliente, MovimentoVenditaCliente,
    NuovoPrezzoCliente, PrezzoCliente, PrezzoSuggerito, RiepilogoCliente,
};
use crate::{AppError, AppState};
use tauri::State;

/// Riepilogo sempre visibile in cima alla scheda cliente: data ultima
/// vendita (da v_storico_vendite), conteggio totale documenti (da
/// `documenti`, stesso criterio non filtrato del tab "Documenti") e
/// fatturato dell'anno solare in corso calcolato solo sulle fatture
/// (tipo_documento='fattura', stato diverso da annullato/bozza).
/// Una sola query per i tre valori insieme.
#[tauri::command]
pub async fn get_riepilogo_cliente(
    cliente_id: i64,
    state: State<'_, AppState>,
) -> Result<RiepilogoCliente, AppError> {
    let row = sqlx::query_as::<_, RiepilogoCliente>(
        "SELECT
            (SELECT MAX(data) FROM v_storico_vendite WHERE cliente_id = ?1) AS ultima_vendita,
            (SELECT COUNT(*) FROM documenti WHERE cliente_id = ?1) AS numero_documenti,
            (SELECT COALESCE(SUM(totale_documento), 0.0) FROM documenti
             WHERE cliente_id = ?1
               AND tipo_documento = 'fattura'
               AND stato NOT IN ('annullato', 'bozza')
               AND strftime('%Y', data) = strftime('%Y', 'now')
            ) AS fatturato_anno_corrente",
    )
    .bind(cliente_id)
    .fetch_one(&state.db)
    .await?;
    Ok(row)
}

/// Elenco completo di tutti i documenti intestati al cliente (tab
/// "Documenti"). Legge direttamente da `documenti`, filtro solo su
/// cliente_id: nessuna esclusione per tipo o stato. E' volutamente
/// diverso da v_storico_vendite, che invece filtra per lo storico prezzi.
#[tauri::command]
pub async fn get_documenti_cliente(
    cliente_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<DocumentoCliente>, AppError> {
    let rows = sqlx::query_as::<_, DocumentoCliente>(
        "SELECT id AS documento_id, tipo_documento, numero, data,
                totale_documento AS totale, stato
         FROM documenti
         WHERE cliente_id = ?
         ORDER BY data DESC, id DESC",
    )
    .bind(cliente_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

/// Aggregato per articolo: prezzo ultimo/min/max/medio ponderato,
/// quantita' totale, numero vendite, data ultima vendita.
/// Alimenta la tab "Articoli venduti" della scheda cliente.
#[tauri::command]
pub async fn get_articoli_venduti_cliente(
    cliente_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<ArticoloVendutoCliente>, AppError> {
    let rows = sqlx::query_as::<_, ArticoloVendutoCliente>(
        "SELECT
            articolo_id,
            codice_articolo,
            descrizione,
            (SELECT v2.prezzo_unitario FROM v_storico_vendite v2
             WHERE v2.cliente_id = v.cliente_id
               AND v2.articolo_id = v.articolo_id
             ORDER BY v2.data DESC, v2.documento_id DESC LIMIT 1
            ) AS prezzo_ultimo,
            MIN(prezzo_unitario) AS prezzo_min,
            MAX(prezzo_unitario) AS prezzo_max,
            SUM(prezzo_unitario * quantita) / SUM(quantita) AS prezzo_medio,
            CAST(SUM(quantita) AS REAL) AS quantita_totale,
            COUNT(*) AS numero_vendite,
            MAX(data) AS ultima_vendita
         FROM v_storico_vendite v
         WHERE cliente_id = ?
         GROUP BY articolo_id, codice_articolo, descrizione
         ORDER BY ultima_vendita DESC",
    )
    .bind(cliente_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

/// Storico completo cronologico del cliente (tab "Movimenti").
#[tauri::command]
pub async fn get_movimenti_cliente(
    cliente_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<MovimentoVenditaCliente>, AppError> {
    let rows = sqlx::query_as::<_, MovimentoVenditaCliente>(
        "SELECT documento_id, tipo_documento, numero_documento, data,
                articolo_id, codice_articolo, descrizione,
                quantita, prezzo_unitario, sconto_perc, totale_riga
         FROM v_storico_vendite
         WHERE cliente_id = ?
         ORDER BY data DESC, documento_id DESC",
    )
    .bind(cliente_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

/// Movimenti di un singolo articolo per un cliente (riga espandibile
/// nella tab "Articoli venduti").
#[tauri::command]
pub async fn get_movimenti_articolo_cliente(
    cliente_id: i64,
    articolo_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<MovimentoVenditaCliente>, AppError> {
    let rows = sqlx::query_as::<_, MovimentoVenditaCliente>(
        "SELECT documento_id, tipo_documento, numero_documento, data,
                articolo_id, codice_articolo, descrizione,
                quantita, prezzo_unitario, sconto_perc, totale_riga
         FROM v_storico_vendite
         WHERE cliente_id = ? AND articolo_id = ?
         ORDER BY data DESC, documento_id DESC",
    )
    .bind(cliente_id)
    .bind(articolo_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

/// Cascata prezzo per una riga documento: dedicato -> storico -> listino.
/// Degrada in modo pulito se cliente_id e' nullo o il cliente non ha
/// dati (cliente occasionale): restituisce Listino, mai un errore.
#[tauri::command]
pub async fn get_prezzo_suggerito(
    cliente_id: Option<i64>,
    ricambio_id: i64,
    state: State<'_, AppState>,
) -> Result<PrezzoSuggerito, AppError> {
    let Some(cid) = cliente_id else {
        return Ok(PrezzoSuggerito::Listino);
    };

    let dedicato = sqlx::query_as::<_, (f64, Option<f64>, Option<String>)>(
        "SELECT prezzo, sconto_perc, note FROM prezzi_cliente
         WHERE cliente_id = ? AND ricambio_id = ?",
    )
    .bind(cid)
    .bind(ricambio_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some((prezzo, sconto_perc, note)) = dedicato {
        return Ok(PrezzoSuggerito::Dedicato { prezzo, sconto_perc, note });
    }

    let storico = sqlx::query_as::<_, (f64, String, String, String)>(
        "SELECT prezzo_unitario, data, tipo_documento, numero_documento
         FROM v_storico_vendite
         WHERE cliente_id = ? AND articolo_id = ?
         ORDER BY data DESC, documento_id DESC LIMIT 1",
    )
    .bind(cid)
    .bind(ricambio_id)
    .fetch_optional(&state.db)
    .await?;

    if let Some((prezzo, data, tipo_documento, numero_documento)) = storico {
        return Ok(PrezzoSuggerito::Storico {
            prezzo,
            data,
            tipo_documento,
            numero_documento,
        });
    }

    Ok(PrezzoSuggerito::Listino)
}

#[tauri::command]
pub async fn list_prezzi_cliente(
    cliente_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<PrezzoCliente>, AppError> {
    let rows = sqlx::query_as::<_, PrezzoCliente>(
        "SELECT pc.id, pc.cliente_id, pc.ricambio_id,
                r.codice_interno, r.descrizione,
                pc.prezzo, pc.sconto_perc, pc.note, pc.aggiornato_il
         FROM prezzi_cliente pc
         JOIN ricambi r ON r.id = pc.ricambio_id
         WHERE pc.cliente_id = ?
         ORDER BY r.codice_interno ASC",
    )
    .bind(cliente_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

#[tauri::command]
pub async fn upsert_prezzo_cliente(
    input: NuovoPrezzoCliente,
    state: State<'_, AppState>,
) -> Result<PrezzoCliente, AppError> {
    if input.prezzo < 0.0 {
        return Err(AppError::Validation(
            "il prezzo non puo' essere negativo".into(),
        ));
    }

    sqlx::query(
        "INSERT INTO prezzi_cliente
            (cliente_id, ricambio_id, prezzo, sconto_perc, note, aggiornato_il)
         VALUES (?, ?, ?, ?, ?, datetime('now'))
         ON CONFLICT(cliente_id, ricambio_id) DO UPDATE SET
            prezzo = excluded.prezzo,
            sconto_perc = excluded.sconto_perc,
            note = excluded.note,
            aggiornato_il = datetime('now')",
    )
    .bind(input.cliente_id)
    .bind(input.ricambio_id)
    .bind(input.prezzo)
    .bind(input.sconto_perc)
    .bind(&input.note)
    .execute(&state.db)
    .await?;

    let row = sqlx::query_as::<_, PrezzoCliente>(
        "SELECT pc.id, pc.cliente_id, pc.ricambio_id,
                r.codice_interno, r.descrizione,
                pc.prezzo, pc.sconto_perc, pc.note, pc.aggiornato_il
         FROM prezzi_cliente pc
         JOIN ricambi r ON r.id = pc.ricambio_id
         WHERE pc.cliente_id = ? AND pc.ricambio_id = ?",
    )
    .bind(input.cliente_id)
    .bind(input.ricambio_id)
    .fetch_one(&state.db)
    .await?;
    Ok(row)
}

#[tauri::command]
pub async fn delete_prezzo_cliente(
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let rows = sqlx::query("DELETE FROM prezzi_cliente WHERE id=?")
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!(
            "prezzo cliente id={id} non trovato"
        )));
    }
    Ok(())
}
