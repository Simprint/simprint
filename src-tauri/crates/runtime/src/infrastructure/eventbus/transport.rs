use super::error::{EventBusError, Result};
use super::message::Message;

#[cfg(windows)]
mod imp {
    use super::*;
    use crate::infrastructure::diagnostics::{log_error, log_info};
    use bytes::BytesMut;
    use std::ffi::OsString;
    use std::sync::Arc;
    use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
    use tokio::sync::Mutex;

    const PIPE_PREFIX: &str = r"\\.\pipe\simprint_";
    const READ_BUFFER_SIZE: usize = 65536;

    pub fn get_pipe_path(env_id: &str) -> String {
        format!("{}{}", PIPE_PREFIX, env_id)
    }

    #[allow(dead_code)]
    pub struct PipeServer {
        env_id: String,
        pipe_path: String,
    }

    #[allow(dead_code)]
    impl PipeServer {
        pub fn new(env_id: &str) -> Self {
            Self {
                env_id: env_id.to_string(),
                pipe_path: get_pipe_path(env_id),
            }
        }

        pub fn env_id(&self) -> &str {
            &self.env_id
        }

        pub fn pipe_path(&self) -> &str {
            &self.pipe_path
        }

        pub async fn accept(&self) -> Result<PipeConnection> {
            let pipe_path_os: OsString = OsString::from(&self.pipe_path);
            let server = ServerOptions::new()
                .first_pipe_instance(true)
                .create(&pipe_path_os)
                .map_err(|error| {
                    log_error("eventbus", format!("Failed to create pipe: {}", error));
                    EventBusError::Connection(format!("failed to create pipe: {}", error))
                })?;

            log_info(
                "eventbus",
                format!("Waiting for connection on: {}", self.pipe_path),
            );

            server.connect().await.map_err(|error| {
                log_error(
                    "eventbus",
                    format!("Failed to accept connection: {}", error),
                );
                EventBusError::Connection(format!("failed to accept connection: {}", error))
            })?;

            log_info(
                "eventbus",
                format!("Client connected on: {}", self.pipe_path),
            );

            Ok(PipeConnection::new(server, self.env_id.clone()))
        }
    }

    pub struct PipeConnection {
        pipe: Arc<NamedPipeServer>,
        env_id: String,
        read_buffer: Arc<Mutex<BytesMut>>,
    }

    impl PipeConnection {
        fn new(pipe: NamedPipeServer, env_id: String) -> Self {
            Self {
                pipe: Arc::new(pipe),
                env_id,
                read_buffer: Arc::new(Mutex::new(BytesMut::with_capacity(READ_BUFFER_SIZE))),
            }
        }

        pub fn env_id(&self) -> &str {
            &self.env_id
        }

        pub async fn send(&self, msg: &Message) -> Result<()> {
            let data = msg.encode()?;

            let mut pos = 0;
            while pos < data.len() {
                self.pipe.writable().await.map_err(|error| {
                    log_error("eventbus", format!("Writable wait failed: {}", error));
                    EventBusError::SendFailed(error.to_string())
                })?;

                match self.pipe.try_write(&data[pos..]) {
                    Ok(n) => pos += n,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                    Err(error) => {
                        log_error("eventbus", format!("Send failed: {}", error));
                        return Err(EventBusError::SendFailed(error.to_string()));
                    }
                }
            }

            Ok(())
        }

        pub async fn recv(&self) -> Result<Message> {
            let mut temp_buf = [0u8; READ_BUFFER_SIZE];

            loop {
                {
                    let mut buffer = self.read_buffer.lock().await;
                    if let Some((msg, consumed)) = Message::try_decode(&buffer)? {
                        let _ = buffer.split_to(consumed);
                        return Ok(msg);
                    }
                }

                self.pipe.readable().await.map_err(|error| {
                    log_error("eventbus", format!("Readable wait failed: {}", error));
                    EventBusError::ReceiveFailed(error.to_string())
                })?;

                match self.pipe.try_read(&mut temp_buf) {
                    Ok(0) => return Err(EventBusError::ReceiveFailed("connection closed".into())),
                    Ok(n) => {
                        let mut buffer = self.read_buffer.lock().await;
                        buffer.extend_from_slice(&temp_buf[..n]);
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => continue,
                    Err(error) => {
                        log_error("eventbus", format!("Read failed: {}", error));
                        return Err(EventBusError::ReceiveFailed(error.to_string()));
                    }
                }
            }
        }

        pub async fn try_recv(&self) -> Result<Option<Message>> {
            {
                let mut buffer = self.read_buffer.lock().await;
                if let Some((msg, consumed)) = Message::try_decode(&buffer)? {
                    let _ = buffer.split_to(consumed);
                    return Ok(Some(msg));
                }
            }

            let mut temp_buf = [0u8; READ_BUFFER_SIZE];
            let n = match self.pipe.try_read(&mut temp_buf) {
                Ok(n) => n,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
                Err(error) => return Err(EventBusError::ReceiveFailed(error.to_string())),
            };

            if n == 0 {
                return Err(EventBusError::ReceiveFailed("connection closed".into()));
            }

            {
                let mut buffer = self.read_buffer.lock().await;
                buffer.extend_from_slice(&temp_buf[..n]);
                if let Some((msg, consumed)) = Message::try_decode(&buffer)? {
                    let _ = buffer.split_to(consumed);
                    return Ok(Some(msg));
                }
            }

            Ok(None)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_pipe_path() {
            let path = get_pipe_path("env_123");
            assert_eq!(path, r"\\.\pipe\simprint_env_123");
        }
    }
}

#[cfg(not(windows))]
#[allow(dead_code)]
mod imp {
    use super::*;

    pub fn get_pipe_path(env_id: &str) -> String {
        format!("simprint_{}", env_id)
    }

    pub struct PipeServer {
        env_id: String,
        pipe_path: String,
    }

    impl PipeServer {
        pub fn new(env_id: &str) -> Self {
            Self {
                env_id: env_id.to_string(),
                pipe_path: get_pipe_path(env_id),
            }
        }

        pub fn env_id(&self) -> &str {
            &self.env_id
        }

        pub fn pipe_path(&self) -> &str {
            &self.pipe_path
        }

        pub async fn accept(&self) -> Result<PipeConnection> {
            Err(EventBusError::Connection(
                "eventbus named pipe transport is only implemented on Windows".into(),
            ))
        }
    }

    #[allow(dead_code)]
    pub struct PipeConnection {
        env_id: String,
    }

    #[allow(dead_code)]
    impl PipeConnection {
        pub fn env_id(&self) -> &str {
            &self.env_id
        }

        pub async fn send(&self, _msg: &Message) -> Result<()> {
            Err(EventBusError::SendFailed(
                "eventbus named pipe transport is unavailable on this platform".into(),
            ))
        }

        pub async fn recv(&self) -> Result<Message> {
            Err(EventBusError::ReceiveFailed(
                "eventbus named pipe transport is unavailable on this platform".into(),
            ))
        }

        pub async fn try_recv(&self) -> Result<Option<Message>> {
            Err(EventBusError::ReceiveFailed(
                "eventbus named pipe transport is unavailable on this platform".into(),
            ))
        }
    }
}

pub use imp::{PipeConnection, PipeServer, get_pipe_path};
