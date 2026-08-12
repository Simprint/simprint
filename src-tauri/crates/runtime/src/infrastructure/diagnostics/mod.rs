use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_log_level(self) -> log::Level {
        match self {
            Self::Debug => log::Level::Debug,
            Self::Info => log::Level::Info,
            Self::Warn => log::Level::Warn,
            Self::Error => log::Level::Error,
        }
    }
}

pub fn unix_now_ms() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as u64,
        Err(_) => 0,
    }
}

pub fn log(level: LogLevel, target: &str, message: impl AsRef<str>) {
    log::log!(target: target, level.as_log_level(), "{}", message.as_ref());
}

pub fn log_debug(target: &str, message: impl AsRef<str>) {
    log(LogLevel::Debug, target, message);
}

pub fn log_info(target: &str, message: impl AsRef<str>) {
    log(LogLevel::Info, target, message);
}

pub fn log_warn(target: &str, message: impl AsRef<str>) {
    log(LogLevel::Warn, target, message);
}

pub fn log_error(target: &str, message: impl AsRef<str>) {
    log(LogLevel::Error, target, message);
}
