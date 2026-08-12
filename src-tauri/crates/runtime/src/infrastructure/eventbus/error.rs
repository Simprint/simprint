use thiserror::Error;

#[derive(Error, Debug)]
pub enum EventBusError {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("not connected: {0}")]
    NotConnected(String),

    #[error("send failed: {0}")]
    SendFailed(String),

    #[error("receive failed: {0}")]
    ReceiveFailed(String),

    #[error("encode error: {0}")]
    Encode(String),

    #[error("decode error: {0}")]
    Decode(String),

    #[error("invalid message: {0}")]
    InvalidMessage(String),

    #[error("unknown topic: {0}")]
    UnknownTopic(u16),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, EventBusError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    Success = 0,
    ConnectionFailed = 1001,
    ConnectionLost = 1002,
    SendFailed = 1003,
    InvalidMessage = 2001,
    UnknownTopic = 2002,
    DecodeFailed = 2003,
    InvalidConfig = 3001,
    PermissionDenied = 3002,
}

impl From<i32> for ErrorCode {
    fn from(code: i32) -> Self {
        match code {
            0 => Self::Success,
            1001 => Self::ConnectionFailed,
            1002 => Self::ConnectionLost,
            1003 => Self::SendFailed,
            2001 => Self::InvalidMessage,
            2002 => Self::UnknownTopic,
            2003 => Self::DecodeFailed,
            3001 => Self::InvalidConfig,
            3002 => Self::PermissionDenied,
            _ => Self::InvalidMessage,
        }
    }
}
