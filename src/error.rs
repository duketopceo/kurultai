use thiserror::Error;

/// Application-wide result type. Domain modules return typed errors here.
pub type Result<T> = std::result::Result<T, KurultaiError>;

#[derive(Debug, Error)]
pub enum KurultaiError {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("security error: {0}")]
    Security(String),

    #[error("connector '{name}': {message}")]
    Connector { name: String, message: String },

    #[error("store error: {0}")]
    Store(String),

    #[error("embed error: {0}")]
    Embed(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("pipeline error: {0}")]
    Pipeline(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl KurultaiError {
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    pub fn security(msg: impl Into<String>) -> Self {
        Self::Security(msg.into())
    }

    pub fn connector(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Connector {
            name: name.into(),
            message: message.into(),
        }
    }
}
