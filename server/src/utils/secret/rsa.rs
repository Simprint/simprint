use std::{fmt::Debug, fs, path::Path};

use base64::Engine;
use rsa::{
    Pkcs1v15Encrypt, RsaPrivateKey,
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
};
use tokio::sync::OnceCell;

#[derive(Debug, Clone)]
pub struct RsaSecret {
    pub private_key: RsaPrivateKey,
    pub public_key: String,
}

impl RsaSecret {
    pub fn new(key_path: &str) -> Result<Self, anyhow::Error> {
        let key_dir = Path::new(key_path);
        let private_key_path = key_dir.join("private_key.pem");
        let public_key_path = key_dir.join("public_key.pem");

        // 如果已经有密钥文件，优先复用，避免每次启动刷新
        if private_key_path.exists() && public_key_path.exists() {
            let private_key_pem = fs::read_to_string(&private_key_path)?;
            let private_key = RsaPrivateKey::from_pkcs1_pem(&private_key_pem)?;
            let public_key_pem = fs::read_to_string(&public_key_path)?;

            return Ok(RsaSecret {
                private_key,
                public_key: public_key_pem,
            });
        }

        // 否则生成新的密钥对
        if !key_dir.exists() {
            fs::create_dir_all(key_dir)?;
        }

        let mut rng = rsa::rand_core::OsRng::default();
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits)?;
        let public_key = private_key.to_public_key();

        let private_key_pem = private_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)?.to_string();
        let public_key_pem = public_key.to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)?;

        fs::write(&private_key_path, &private_key_pem)?;
        fs::write(&public_key_path, &public_key_pem)?;

        Ok(RsaSecret {
            private_key,
            public_key: public_key_pem,
        })
    }

    /// 获取公钥对
    pub fn get_public_key(&self) -> String {
        self.public_key.clone()
    }

    /// 获取私钥对
    pub fn get_private_key(&self) -> RsaPrivateKey {
        self.private_key.clone()
    }

    /// 解密为默认的&[u8]
    pub fn decrypt(&self, data: &str) -> Result<Vec<u8>, anyhow::Error> {
        // base64 解码
        let data = base64::engine::general_purpose::STANDARD.decode(data).map_err(|e| {
            tracing::error!("Failed to decode base64 data: {:?}", e);
            anyhow::anyhow!("Failed to decode base64 data: {:?}", e)
        })?;

        let private_key = &self.private_key;
        let decrypted_data = private_key.decrypt(Pkcs1v15Encrypt, &data)?;

        Ok(decrypted_data)
    }

    /// 使用公钥加密
    pub fn encrypt(&self, data: &[u8]) -> Result<String, anyhow::Error> {
        let public_key = &self.private_key.to_public_key();

        let mut rng = rsa::rand_core::OsRng::default();
        let encrypted_data = public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data).map_err(|e| {
            tracing::error!("Failed to encrypt data: {:?}", e);
            anyhow::anyhow!("Failed to encrypt data: {:?}", e)
        })?;

        let encoded_data = base64::engine::general_purpose::STANDARD.encode(&encrypted_data);

        Ok(encoded_data)
    }

    /// 使用公钥加密, 公钥接收到的公钥
    pub fn encrypt_with_public_key(
        &self,
        data: &[u8],
        public_key: &str,
    ) -> Result<String, anyhow::Error> {
        let public_key = rsa::RsaPublicKey::from_pkcs1_pem(public_key).map_err(|e| {
            tracing::error!("Failed to parse public key: {:?}", e);
            anyhow::anyhow!("Failed to parse public key: {:?}", e)
        })?;

        let mut rng = rsa::rand_core::OsRng::default();
        let encrypted_data = public_key.encrypt(&mut rng, Pkcs1v15Encrypt, data).map_err(|e| {
            tracing::error!("Failed to encrypt data: {:?}", e);
            anyhow::anyhow!("Failed to encrypt data: {:?}", e)
        })?;

        let encoded_data = base64::engine::general_purpose::STANDARD.encode(&encrypted_data);

        Ok(encoded_data)
    }
}

/// 全局的 RSA 密钥对
pub static RSA_SECRET_INSTANCE: OnceCell<RsaSecret> = OnceCell::const_new();

/// 初始化全局的 RSA 密钥对
pub async fn init_rsa_secret(key_path: &str) -> &'static RsaSecret {
    RSA_SECRET_INSTANCE
        .get_or_init(|| async { RsaSecret::new(key_path).unwrap() })
        .await
}

/// 获取全局的 RSA 密钥管理实例
pub fn get_rsa_secret_instance() -> &'static RsaSecret {
    match RSA_SECRET_INSTANCE.get() {
        Some(secret_instance) => secret_instance,
        None => {
            tracing::debug!("RSA_SECRET_INSTANCE is not initialized");
            std::process::exit(-1);
        }
    }
}

#[cfg(test)]
mod tests {

    use serde::{Deserialize, Serialize};

    use super::*;

    #[tokio::test]
    async fn test_rsa_secret_build() {
        #[derive(Serialize, Deserialize, Clone, Debug)]
        struct TestStruct {
            name: String,
            age: u32,
        }

        init_rsa_secret("../../assets/secret").await;

        let secret_instance = get_rsa_secret_instance();

        let data = TestStruct {
            name: "test".to_string(),
            age: 18,
        };

        let data = serde_json::to_value(data).unwrap().to_string();
        let data = data.as_bytes();

        let encrypt_data = secret_instance.encrypt(&data).unwrap();
        eprintln!("public_key: {:?}", encrypt_data);

        let res = secret_instance.decrypt(&encrypt_data).unwrap();
        eprintln!("data: {:?}", res);
    }
}
