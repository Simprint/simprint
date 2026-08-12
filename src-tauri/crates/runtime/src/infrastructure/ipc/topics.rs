#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topic {
    Handshake,
    Ping,
    QueryState,
    QueryHealth,
    Shutdown,
    InitializeContext,
    DestroyContext,
    EnvironmentCommand,
    SyncCommand,
    AuthCommand,
    RuntimeEvent,
    Unknown(u16),
}

impl From<u16> for Topic {
    fn from(value: u16) -> Self {
        match value {
            0x0001 => Self::Handshake,
            0x0002 => Self::Ping,
            0x0003 => Self::QueryState,
            0x0004 => Self::QueryHealth,
            0x0005 => Self::Shutdown,
            0x0100 => Self::InitializeContext,
            0x0101 => Self::DestroyContext,
            0x0200 => Self::EnvironmentCommand,
            0x0300 => Self::SyncCommand,
            0x0400 => Self::AuthCommand,
            0x7F00 => Self::RuntimeEvent,
            unknown => Self::Unknown(unknown),
        }
    }
}

impl From<Topic> for u16 {
    fn from(topic: Topic) -> Self {
        match topic {
            Topic::Handshake => 0x0001,
            Topic::Ping => 0x0002,
            Topic::QueryState => 0x0003,
            Topic::QueryHealth => 0x0004,
            Topic::Shutdown => 0x0005,
            Topic::InitializeContext => 0x0100,
            Topic::DestroyContext => 0x0101,
            Topic::EnvironmentCommand => 0x0200,
            Topic::SyncCommand => 0x0300,
            Topic::AuthCommand => 0x0400,
            Topic::RuntimeEvent => 0x7F00,
            Topic::Unknown(value) => value,
        }
    }
}
