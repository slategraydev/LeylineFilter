use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Audio device error: {0}")]
    DeviceError(String),

    #[error("Stream error: {0}")]
    StreamError(#[from] cpal::StreamError),

    #[error("Devices error: {0}")]
    DevicesError(#[from] cpal::DevicesError),

    #[error("Build stream error: {0}")]
    BuildStreamError(#[from] cpal::BuildStreamError),

    #[error("Play stream error: {0}")]
    PlayStreamError(#[from] cpal::PlayStreamError),

    #[error("Default stream config error: {0}")]
    DefaultStreamConfigError(#[from] cpal::DefaultStreamConfigError),

    #[error("Resampler error: {0}")]
    ResamplerError(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Lock contention error: {0}")]
    LockError(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
