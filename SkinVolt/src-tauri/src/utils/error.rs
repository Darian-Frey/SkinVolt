use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkinVoltError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Steam API error: {0}")]
    Steam(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
