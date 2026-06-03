use crate::{AppError, AppState};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct RisultatoRicerca {
    pub tipo: String,
    pub id: i64,
    pub titolo: String,
    pub sottotitolo: Option<String>,
}

#[tauri::command]
pub async fn search_global(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<RisultatoRicerca>, AppError> {
    let trimmed = query.trim();
    if trimmed.len() < 2 {
        return Ok(vec![]);
    }

    let pattern = format!("%{}%", trimmed);
    let mut risultati: Vec<RisultatoRicerca> = Vec::new();

    let clienti = sqlx::query_as::<_, (i64, String, Option<String>)>(
        "SELECT id, ragione_sociale, partita_iva FROM clienti
         WHERE ragione_sociale LIKE ? OR partita_iva LIKE ?
            OR codice_fiscale LIKE ? OR email LIKE ?
         LIMIT 5",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&state.db)
    .await?;

    for (id, nome, piva) in clienti {
        risultati.push(RisultatoRicerca {
            tipo: "cliente".into(),
            id,
            titolo: nome,
            sottotitolo: piva,
        });
    }

    let ricambi = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, codice_interno, descrizione FROM ricambi
         WHERE codice_interno LIKE ? OR descrizione LIKE ? OR codice_oem LIKE ?
         LIMIT 5",
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&state.db)
    .await?;

    for (id, codice, desc) in ricambi {
        risultati.push(RisultatoRicerca {
            tipo: "ricambio".into(),
            id,
            titolo: desc,
            sottotitolo: Some(codice),
        });
    }

    let documenti = sqlx::query_as::<_, (i64, String, String, f64)>(
        "SELECT id, tipo_documento, numero, totale_documento FROM documenti
         WHERE numero LIKE ? OR note LIKE ?
         LIMIT 5",
    )
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(&state.db)
    .await?;

    for (id, tipo, numero, totale) in documenti {
        let tipo_label = match tipo.as_str() {
            "fattura" => "Fattura",
            "preventivo" => "Preventivo",
            "ddt" => "DDT",
            "nota_credito" => "Nota credito",
            other => other,
        };
        risultati.push(RisultatoRicerca {
            tipo: "documento".into(),
            id,
            titolo: format!("{} n. {}", tipo_label, numero),
            sottotitolo: Some(format!("€ {:.2}", totale)),
        });
    }

    Ok(risultati)
}
