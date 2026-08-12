use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpcError {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("connection closed")]
    ConnectionClosed,

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

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, IpcError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    Success = 0,
    ConnectionFailed = 1001,
    ConnectionClosed = 1002,
    SendFailed = 1003,
    InvalidMessage = 2001,
    UnknownTopic = 2002,
    DecodeFailed = 2003,
    HandshakeRequired = 2004,
    InvalidState = 3001,
    AlreadyInitialized = 3002,
    NotInitialized = 3003,
    ModuleFailed = 4001,
    NotImplemented = 4002,
    InternalError = 5000,
}

impl ErrorCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}
