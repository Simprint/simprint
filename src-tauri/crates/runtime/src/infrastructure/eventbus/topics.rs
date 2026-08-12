use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum Topic {
    Handshake = 0x0001,
    Heartbeat = 0x0002,
    Disconnect = 0x0003,
    ConfigInit = 0x0100,
    ConfigUpdate = 0x0101,
    FingerprintApply = 0x0200,
    FingerprintQuery = 0x0201,
    ProxySet = 0x0300,
    ProxyBypass = 0x0301,
    RpaCommand = 0x0400,
    RpaResult = 0x0401,
    RpaEvent = 0x0402,
    PageLoad = 0x0500,
    PageClose = 0x0501,
    NavigationStart = 0x0502,
    AuthRequest = 0x0600,
    AuthResponse = 0x0601,
    WindowSetBounds = 0x0700,
    SyncInputEvent = 0x0800,
    SyncRole = 0x0801,
    SyncInputDebug = 0x0802,
    SyncPaste = 0x0803,
    LaunchConfig = 0x0900,
}

impl From<u16> for Topic {
    fn from(value: u16) -> Self {
        match value {
            0x0001 => Self::Handshake,
            0x0002 => Self::Heartbeat,
            0x0003 => Self::Disconnect,
            0x0100 => Self::ConfigInit,
            0x0101 => Self::ConfigUpdate,
            0x0200 => Self::FingerprintApply,
            0x0201 => Self::FingerprintQuery,
            0x0300 => Self::ProxySet,
            0x0301 => Self::ProxyBypass,
            0x0400 => Self::RpaCommand,
            0x0401 => Self::RpaResult,
            0x0402 => Self::RpaEvent,
            0x0500 => Self::PageLoad,
            0x0501 => Self::PageClose,
            0x0502 => Self::NavigationStart,
            0x0600 => Self::AuthRequest,
            0x0601 => Self::AuthResponse,
            0x0700 => Self::WindowSetBounds,
            0x0800 => Self::SyncInputEvent,
            0x0801 => Self::SyncRole,
            0x0802 => Self::SyncInputDebug,
            0x0803 => Self::SyncPaste,
            0x0900 => Self::LaunchConfig,
            _ => Self::Handshake,
        }
    }
}
