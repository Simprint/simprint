use crate::infrastructure::ipc::{ErrorCode, IpcError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("invalid runtime state: {0}")]
    InvalidState(String),

    #[error("runtime context already initialized")]
    AlreadyInitialized,

    #[error("runtime context is not initialized")]
    NotInitialized,

    #[error("module '{module}' failed during '{action}': {message}")]
    ModuleLifecycle {
        module: &'static str,
        action: &'static str,
        message: String,
    },

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("ipc error: {0}")]
    Ipc(#[from] IpcError),

    #[error("eventbus error: {0}")]
    EventBus(#[from] crate::infrastructure::eventbus::EventBusError),

    #[error("internal error: {0}")]
    Internal(String),
}

impl RuntimeError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::InvalidState(_) => ErrorCode::InvalidState,
            Self::AlreadyInitialized => ErrorCode::AlreadyInitialized,
            Self::NotInitialized => ErrorCode::NotInitialized,
            Self::ModuleLifecycle { .. } => ErrorCode::ModuleFailed,
            Self::Serialization(_) => ErrorCode::InternalError,
            Self::Ipc(IpcError::Connection(_)) => ErrorCode::ConnectionFailed,
            Self::Ipc(IpcError::ConnectionClosed) => ErrorCode::ConnectionClosed,
            Self::Ipc(IpcError::SendFailed(_)) => ErrorCode::SendFailed,
            Self::Ipc(IpcError::ReceiveFailed(_)) => ErrorCode::ConnectionClosed,
            Self::Ipc(IpcError::Encode(_)) => ErrorCode::InternalError,
            Self::Ipc(IpcError::Decode(_)) => ErrorCode::DecodeFailed,
            Self::Ipc(IpcError::InvalidMessage(_)) => ErrorCode::InvalidMessage,
            Self::Ipc(IpcError::Serialization(_)) => ErrorCode::InternalError,
            Self::Ipc(IpcError::Io(_)) => ErrorCode::InternalError,
            Self::EventBus(_) => ErrorCode::InternalError,
            Self::Internal(_) => ErrorCode::InternalError,
        }
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
