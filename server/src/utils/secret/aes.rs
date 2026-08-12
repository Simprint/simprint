use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use anyhow::Context;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use std::fmt::Debug;

const AES_KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;

/// key本身是字节数组, 先将字节数组转换成base64编码得字符串，然后再通过非对称公钥加密再转换成base64编码的字符串
#[derive(Debug, Clone, Default)]
pub struct AesSecret {
    key: Key<Aes256Gcm>, // 直接存储 GenericArray 以避免重复转换
}

impl AesSecret {
    /// 生成一个新的 AES 密钥实例
    pub fn new() -> Self {
        let key_bytes = Aes256Gcm::generate_key(&mut OsRng);
        AesSecret { key: key_bytes }
    }

    /// 从已有的密钥字节创建 AesSecret 实例
    /// 输入的 key_bytes 必须是正确的长度 (例如 32 字节 for AES-256)
    pub fn from_bytes(key_bytes: &[u8]) -> Result<Self, anyhow::Error> {
        if key_bytes.len() != AES_KEY_SIZE {
            return Err(anyhow::anyhow!(
                "Invalid key size: expected {}, got {}",
                AES_KEY_SIZE,
                key_bytes.len()
            ));
        }
        Ok(AesSecret {
            key: Key::<Aes256Gcm>::clone_from_slice(key_bytes),
        })
    }

    /// 获取 AES 密钥的 Base64 编码字符串
    pub fn get_key_as_base64(&self) -> String {
        BASE64_STANDARD.encode(self.key.as_slice())
    }

    /// 使用 AES 密钥加密数据
    /// 返回: 加密后的数据 Vec<u8>与Nonce Vec<u8>拼接的 Vec<u8>
    pub fn encrypt(&self, data: &[u8]) -> Result<String, anyhow::Error> {
        let cipher = Aes256Gcm::new(&self.key);
        // 为每次加密生成新的、唯一的 Nonce
        let nonce_bytes = Aes256Gcm::generate_nonce(&mut OsRng); // 生成随机 Nonce
        let nonce_instance = Nonce::from_slice(nonce_bytes.as_slice());

        let ciphertext = cipher.encrypt(nonce_instance, data).map_err(|e| {
            tracing::error!("Failed to encrypt data: {:?}", e);
            // 使用 context 来添加上下文信息
            anyhow::anyhow!("Failed to encrypt data: {:?}", e)
        })?;

        let mut result = Vec::with_capacity(ciphertext.len() + NONCE_SIZE);
        result.extend_from_slice(nonce_bytes.as_slice());
        result.extend_from_slice(&ciphertext);

        // 将结果转换为 Base64 编码字符串
        let result = BASE64_STANDARD.encode(&result);

        Ok(result)
    }

    /// 使用 AES 密钥和提供的 Nonce 解密数据
    pub fn decrypt(&self, base64_data: &str) -> Result<Vec<u8>, anyhow::Error> {
        // 先对Base64编码的字符串进行解码
        let data = BASE64_STANDARD.decode(base64_data).context("Failed to decode base64 data")?;

        if data.len() < NONCE_SIZE {
            return Err(anyhow::anyhow!("Invalid data"));
        }

        let (nonce, ciphertext) = data.split_at(NONCE_SIZE);
        let cipher = Aes256Gcm::new(&self.key);
        let nonce_instance = Nonce::from_slice(nonce);

        cipher.decrypt(nonce_instance, ciphertext).map_err(|e| {
            tracing::error!("Failed to decrypt data: {:?}", e);
            anyhow::anyhow!("Failed to decrypt data: {:?}", e)
        })
    }
}

/// 从 Base64 编码的密钥字符串创建 AesSecret
impl TryFrom<&str> for AesSecret {
    type Error = anyhow::Error;

    fn try_from(key_base64: &str) -> Result<Self, Self::Error> {
        let decoded_key_bytes = BASE64_STANDARD
            .decode(key_base64)
            .context("Failed to decode base64 key string")?;

        Self::from_bytes(&decoded_key_bytes)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_encrypt_decrypt_workflow() {
        let secret = AesSecret::new();
        let plaintext = b"Hello, secure world of AES-GCM!"; // 字节数组

        // 加密
        let encrypt_str = secret.encrypt(plaintext).expect("Encryption failed");
        println!("encrypt_str: {}", encrypt_str);

        // 解密
        let decrypted_bytes = secret.decrypt(&encrypt_str).unwrap();

        // 将解密后的字节数组转换回字符串
        let original_str = String::from_utf8(decrypted_bytes).unwrap();
        println!("解密后的数据: {}", original_str);
    }

    #[test]
    fn test_encrypt_decrypt_json_workflow() {
        let secret = AesSecret::new();

        // 创建一个要序列化的结构体
        #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
        struct TestData {
            message: String,
            value: i32,
        }

        let test_data = TestData {
            message: "Hello, secure world of AES-GCM!".to_string(),
            value: 42,
        };

        // 序列化为JSON
        let json_bytes = serde_json::to_vec(&test_data).expect("Serialization failed");

        // 加密JSON数据
        let encrypted = secret.encrypt(&json_bytes).expect("Encryption failed");
        println!("Encrypted: {}", encrypted);

        // 解密
        let decrypted = secret.decrypt(&encrypted).expect("Decryption failed");

        // 从JSON反序列化
        let restored_data: TestData =
            serde_json::from_slice(&decrypted).expect("Deserialization failed");
        println!("Restored: {:?}", restored_data);

        // 确认数据一致
        assert_eq!(test_data, restored_data);
    }

    /// 测试加密流程
    #[test]
    fn test_login_payload_encrypt() {
        use rsa::pkcs1::DecodeRsaPublicKey;

        let secret = AesSecret::new();
        let key = secret.get_key_as_base64();

        // 测试通过base64编码的字符串创建AesSecret实例
        // let _ = AesSecret::try_from(key.as_str()).expect("创建AES实例失败");

        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        struct LoginRequest {
            email: String,
            password: String,
            public_secret_key: String,
        }
        let payload = LoginRequest {
            email: "liusnew@gmail.com".to_string(),
            password: "liusNew57~".to_string(),
            public_secret_key: "-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEAseI9vA7iTxOMb5Y2xCL7BOGr1by9qEH4EfP9Bj90gxDmY8yRsVK/\no2g+i95oQxzdvdvpPAocKlQv2FEbZaqFr2Q4vy1cLrM0B1NTZl1/hGcmPSofLT9g\nmnjzP60ikY40Dxq+YXAxXZ4s2M+9thNnFr4OydacHEPEkTklcBQBglopSXc1yqHU\nARyCQ3/VxQrfh215vIPgMg2f6PH741zXFaIJjucXR8wJVySo7aZhlBTOVz5GzV0b\naWh31zA47ivXh84OXIEI+CKDUSnvsa8SCRMRs8LgaO1Xktv4yCfHHpo8Zoy+KdW0\nLJxP11G+3f9RsYRpjdmsleyHuYYS07suqwIDAQAB\n-----END RSA PUBLIC KEY-----\n".to_string(),
        };

        let json_bytes = serde_json::to_vec(&payload).expect("Serialization failed");
        let encrypted = secret.encrypt(&json_bytes).expect("Encryption failed");

        // 下面连续的代码对应： crate::secret::rsa::get_rsa_secret_instance()
        let public_key_str =
            std::fs::read("../../assets/secret/public_key.pem").expect("读取公钥失败");
        let public_key = String::from_utf8(public_key_str).expect("转换公钥失败");
        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(&public_key)
            .map_err(|e| {
                tracing::error!("Failed to parse public key: {:?}", e);
                anyhow::anyhow!("Failed to parse public key: {:?}", e)
            })
            .unwrap();
        let mut rng = rsa::rand_core::OsRng::default();
        let encrypted_data = public_key
            .encrypt(&mut rng, rsa::Pkcs1v15Encrypt, key.as_bytes())
            .map_err(|e| {
                tracing::error!("Failed to encrypt data: {:?}", e);
                anyhow::anyhow!("Failed to encrypt data: {:?}", e)
            })
            .unwrap();

        // 通过一个非对称公钥加密key
        let encrypted_key = base64::engine::general_purpose::STANDARD.encode(&encrypted_data);

        let result = serde_json::json!({
            "data": encrypted,
            "encrypted": true,
            "key": encrypted_key,
        });
        let result = serde_json::to_string_pretty(&result).unwrap();
        eprintln!("{}", result);
    }
}
