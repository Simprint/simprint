use super::error::{ErrorCode, EventBusError, Result};
use super::topics::Topic;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

const MAGIC: [u8; 4] = [0x53, 0x49, 0x4D, 0x00];
pub const VERSION: u8 = 0x01;
const HEADER_SIZE: usize = 9;
const MIN_PAYLOAD_SIZE: usize = 15;

static MSG_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

fn next_msg_id() -> u32 {
    MSG_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    Request = 1,
    Response = 2,
    Event = 3,
}

impl From<u8> for MessageType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Request,
            2 => Self::Response,
            3 => Self::Event,
            _ => Self::Event,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Message {
    pub msg_id: u32,
    pub msg_type: MessageType,
    pub topic: Topic,
    pub error_code: i32,
    pub data: Vec<u8>,
}

impl Message {
    pub fn request(topic: Topic, data: Vec<u8>) -> Self {
        Self {
            msg_id: next_msg_id(),
            msg_type: MessageType::Request,
            topic,
            error_code: 0,
            data,
        }
    }

    pub fn event(topic: Topic, data: Vec<u8>) -> Self {
        Self {
            msg_id: next_msg_id(),
            msg_type: MessageType::Event,
            topic,
            error_code: 0,
            data,
        }
    }

    pub fn response(request_id: u32, topic: Topic, error_code: ErrorCode, data: Vec<u8>) -> Self {
        Self {
            msg_id: request_id,
            msg_type: MessageType::Response,
            topic,
            error_code: error_code as i32,
            data,
        }
    }

    pub fn success_response(request_id: u32, topic: Topic, data: Vec<u8>) -> Self {
        Self::response(request_id, topic, ErrorCode::Success, data)
    }

    pub fn error_response(request_id: u32, topic: Topic, error_code: ErrorCode) -> Self {
        Self::response(request_id, topic, error_code, vec![])
    }

    pub fn encode(&self) -> Result<Bytes> {
        let payload_len = MIN_PAYLOAD_SIZE + self.data.len();
        let mut payload = BytesMut::with_capacity(payload_len);

        payload.put_u32_le(self.msg_id);
        payload.put_u8(self.msg_type as u8);
        payload.put_u16_le(self.topic as u16);
        payload.put_i32_le(self.error_code);
        payload.put_u32_le(self.data.len() as u32);
        payload.put_slice(&self.data);

        let mut buffer = BytesMut::with_capacity(HEADER_SIZE + payload.len());
        buffer.put_slice(&MAGIC);
        buffer.put_u8(VERSION);
        buffer.put_u32_le(payload.len() as u32);
        buffer.put_slice(&payload);

        Ok(buffer.freeze())
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(EventBusError::Decode("data too short for header".into()));
        }

        let mut buffer = data;

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&buffer[..4]);
        buffer.advance(4);
        if magic != MAGIC {
            return Err(EventBusError::Decode("invalid magic number".into()));
        }

        let version = buffer.get_u8();
        if version != VERSION {
            return Err(EventBusError::Decode(format!(
                "unsupported version: {}",
                version
            )));
        }

        let payload_len = buffer.get_u32_le() as usize;
        if buffer.len() < payload_len {
            return Err(EventBusError::Decode("incomplete payload".into()));
        }
        if payload_len < MIN_PAYLOAD_SIZE {
            return Err(EventBusError::Decode("payload too short".into()));
        }

        let msg_id = buffer.get_u32_le();
        let msg_type = MessageType::from(buffer.get_u8());
        let topic = Topic::from(buffer.get_u16_le());
        let error_code = buffer.get_i32_le();
        let data_len = buffer.get_u32_le() as usize;
        if buffer.len() < data_len {
            return Err(EventBusError::Decode("data length mismatch".into()));
        }

        let mut msg_data = vec![0u8; data_len];
        msg_data.copy_from_slice(&buffer[..data_len]);

        Ok(Self {
            msg_id,
            msg_type,
            topic,
            error_code,
            data: msg_data,
        })
    }

    pub fn try_decode(data: &[u8]) -> Result<Option<(Self, usize)>> {
        if data.len() < HEADER_SIZE {
            return Ok(None);
        }

        if &data[..4] != MAGIC {
            return Err(EventBusError::Decode("invalid magic number".into()));
        }

        let payload_len = u32::from_le_bytes([data[5], data[6], data[7], data[8]]) as usize;
        let total_len = HEADER_SIZE + payload_len;
        if data.len() < total_len {
            return Ok(None);
        }

        let message = Self::decode(&data[..total_len])?;
        Ok(Some((message, total_len)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeData {
    pub version: u8,
    pub env_id: String,
    pub client_type: String,
}

impl HandshakeData {
    pub fn browser(env_id: String) -> Self {
        Self {
            version: VERSION,
            env_id,
            client_type: "browser".into(),
        }
    }

    pub fn tauri_response() -> Self {
        Self {
            version: VERSION,
            env_id: String::new(),
            client_type: "tauri".into(),
        }
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|error| EventBusError::Serialization(error.to_string()))
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(|error| EventBusError::Decode(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_encode_decode() {
        let message = Message::event(Topic::Handshake, b"hello".to_vec());
        let encoded = message.encode().unwrap();
        let decoded = Message::decode(&encoded).unwrap();

        assert_eq!(decoded.msg_id, message.msg_id);
        assert_eq!(decoded.topic, Topic::Handshake);
        assert_eq!(decoded.data, b"hello".to_vec());
    }

    #[test]
    fn test_handshake_data() {
        let data = HandshakeData::browser("env_123".into());
        let bytes = data.to_bytes().unwrap();
        let decoded = HandshakeData::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.env_id, "env_123");
        assert_eq!(decoded.client_type, "browser");
    }
}
