use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    #[error("Errore database: {0}")]
    Database(String),

    #[error("Record non trovato: {0}")]
    NotFound(String),

    #[error("Errore di validazione: {0}")]
    Validation(String),

    #[error("Errore interno: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => AppError::NotFound("record non trovato".into()),
            _ => AppError::Database(e.to_string()),
        }
    }
}
