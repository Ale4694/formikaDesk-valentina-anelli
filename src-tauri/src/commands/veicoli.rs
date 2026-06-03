use crate::{AppError, AppState};
use crate::models::{NuovoVeicolo, Veicolo};
use tauri::State;

#[tauri::command]
pub async fn get_veicoli_cliente(
    cliente_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<Veicolo>, AppError> {
    let veicoli = sqlx::query_as::<_, Veicolo>(
        "SELECT * FROM veicoli WHERE cliente_id=? ORDER BY targa ASC",
    )
    .bind(cliente_id)
    .fetch_all(&state.db)
    .await?;
    Ok(veicoli)
}

#[tauri::command]
pub async fn create_veicolo(
    veicolo: NuovoVeicolo,
    state: State<'_, AppState>,
) -> Result<Veicolo, AppError> {
    if veicolo.targa.trim().is_empty() {
        return Err(AppError::Validation("targa obbligatoria".into()));
    }
    let id = sqlx::query(
        "INSERT INTO veicoli (cliente_id, targa, marca, modello, anno, cilindrata, carburante, km_attuali, note)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(veicolo.cliente_id)
    .bind(veicolo.targa.trim().to_uppercase())
    .bind(&veicolo.marca)
    .bind(&veicolo.modello)
    .bind(veicolo.anno)
    .bind(&veicolo.cilindrata)
    .bind(&veicolo.carburante)
    .bind(veicolo.km_attuali)
    .bind(&veicolo.note)
    .execute(&state.db)
    .await?
    .last_insert_rowid();

    let v = sqlx::query_as::<_, Veicolo>("SELECT * FROM veicoli WHERE id=?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    Ok(v)
}

#[tauri::command]
pub async fn update_veicolo(
    id: i64,
    veicolo: NuovoVeicolo,
    state: State<'_, AppState>,
) -> Result<Veicolo, AppError> {
    if veicolo.targa.trim().is_empty() {
        return Err(AppError::Validation("targa obbligatoria".into()));
    }
    let rows = sqlx::query(
        "UPDATE veicoli SET targa=?, marca=?, modello=?, anno=?, cilindrata=?,
         carburante=?, km_attuali=?, note=?, updated_at=datetime('now')
         WHERE id=?",
    )
    .bind(veicolo.targa.trim().to_uppercase())
    .bind(&veicolo.marca)
    .bind(&veicolo.modello)
    .bind(veicolo.anno)
    .bind(&veicolo.cilindrata)
    .bind(&veicolo.carburante)
    .bind(veicolo.km_attuali)
    .bind(&veicolo.note)
    .bind(id)
    .execute(&state.db)
    .await?
    .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("veicolo id={id} non trovato")));
    }
    let v = sqlx::query_as::<_, Veicolo>("SELECT * FROM veicoli WHERE id=?")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    Ok(v)
}

#[tauri::command]
pub async fn delete_veicolo(id: i64, state: State<'_, AppState>) -> Result<(), AppError> {
    let rows = sqlx::query("DELETE FROM veicoli WHERE id=?")
        .bind(id)
        .execute(&state.db)
        .await?
        .rows_affected();
    if rows == 0 {
        return Err(AppError::NotFound(format!("veicolo id={id} non trovato")));
    }
    Ok(())
}
