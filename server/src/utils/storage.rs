use std::{path::Path, sync::Arc};

pub mod get_objects;
pub mod put_objects;
pub mod remove_objects;

use minio::s3::{client::ClientBuilder, creds::StaticProvider, http::BaseUrl};
use tokio::sync::OnceCell;

/// S3-compatible object storage client
pub struct ObjectStorage {
    pub client: minio::s3::client::Client,
}

impl ObjectStorage {
    /// 创建一个新的对象存储客户端
    ///
    /// # Arguments
    /// - `base_url`: 对象存储服务的基础URL。
    /// - `access_key`: 对象存储认证的访问密钥。
    /// - `secret_key`: 对象存储认证的密钥。
    /// - `session_token`: 可选的临时凭证会话令牌。
    /// - `cert_path`: 可选的SSL证书文件路径。
    pub fn new(
        base_url: &str,
        access_key: &str,
        secret_key: &str,
        session_token: Option<&str>,
        cert_path: Option<&Path>,
    ) -> Result<Self, anyhow::Error> {
        let base_url = base_url.parse::<BaseUrl>()?;
        let static_provider = StaticProvider::new(access_key, secret_key, session_token);

        let client = ClientBuilder::new(base_url)
            .provider(Some(Box::new(static_provider)))
            .ssl_cert_file(cert_path)
            .build()?;

        Ok(Self { client })
    }

    // 获取底层对象存储客户端
    pub fn get_client(&self) -> &minio::s3::client::Client {
        &self.client
    }
}

static STORAGE: OnceCell<Arc<ObjectStorage>> = OnceCell::const_new();

/// 初始化对象存储客户端
pub async fn init_storage(
    base_url: &str,
    access_key: &str,
    secret_key: &str,
    session_token: Option<&str>,
    cert_path: Option<&Path>,
) -> Result<&'static Arc<ObjectStorage>, anyhow::Error> {
    Ok(STORAGE
        .get_or_init(|| async {
            let storage = match ObjectStorage::new(
                base_url,
                access_key,
                secret_key,
                session_token,
                cert_path,
            )
            {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to initialize the storage client: {:?}", e);
                    std::process::exit(-1);
                }
            };

            Arc::new(storage)
        })
        .await)
}

/// 获取对象存储客户端
pub fn get_storage_client() -> Result<&'static minio::s3::client::Client, anyhow::Error> {
    match STORAGE.get() {
        Some(storage) => Ok(storage.get_client()),
        None => Err(anyhow::anyhow!("storage client not initialized")),
    }
}
