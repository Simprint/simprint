use std::{
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use bytes::Bytes;
use minio::s3::{
    client::ClientBuilder, creds::StaticProvider, http::BaseUrl, segmented_bytes::SegmentedBytes,
    types::S3Api,
};
use tokio::sync::OnceCell;

use crate::utils::{StorageBackend, StorageConfig};

pub mod get_objects;
pub mod put_objects;
pub mod remove_objects;

pub struct ObjectStorage {
    backend: ObjectStorageBackend,
}

enum ObjectStorageBackend {
    Local { root: PathBuf },
    S3 { client: minio::s3::client::Client },
}

impl ObjectStorage {
    async fn new(config: &StorageConfig) -> Result<Self, anyhow::Error> {
        let backend = match config.backend {
            StorageBackend::Local => {
                let root = config.local_root().to_path_buf();
                tokio::fs::create_dir_all(&root).await?;
                ObjectStorageBackend::Local { root }
            }
            StorageBackend::S3 => {
                let base_url = config.endpoint.parse::<BaseUrl>()?;
                let provider =
                    StaticProvider::new(&config.access_key, &config.secret_access_key, None);
                let client =
                    ClientBuilder::new(base_url).provider(Some(Box::new(provider))).build()?;
                ObjectStorageBackend::S3 { client }
            }
        };

        Ok(Self { backend })
    }

    pub async fn put_object(
        &self,
        bucket: &str,
        object_path: &str,
        data: Bytes,
    ) -> Result<(), anyhow::Error> {
        match &self.backend {
            ObjectStorageBackend::Local { root } => {
                let bucket_root = safe_join(root, bucket)?;
                let file_path = safe_join(&bucket_root, object_path)?;
                if let Some(parent) = file_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::write(file_path, data).await?;
            }
            ObjectStorageBackend::S3 { client } => {
                let data = SegmentedBytes::from(data);
                client.put_object(bucket, object_path, data).send().await?;
            }
        }
        Ok(())
    }
}

static STORAGE: OnceCell<Arc<ObjectStorage>> = OnceCell::const_new();

pub async fn init_storage(
    config: &StorageConfig,
) -> Result<&'static Arc<ObjectStorage>, anyhow::Error> {
    STORAGE
        .get_or_try_init(|| async { ObjectStorage::new(config).await.map(Arc::new) })
        .await
}

pub fn get_storage() -> Result<&'static Arc<ObjectStorage>, anyhow::Error> {
    STORAGE.get().ok_or_else(|| anyhow::anyhow!("storage not initialized"))
}

fn safe_join(root: &Path, relative_path: &str) -> Result<PathBuf, anyhow::Error> {
    let mut output = root.to_path_buf();
    for component in Path::new(relative_path).components() {
        match component {
            Component::Normal(segment) => output.push(segment),
            Component::CurDir => {}
            _ => return Err(anyhow::anyhow!("invalid storage path: {relative_path}")),
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{ObjectStorage, safe_join};
    use crate::utils::{StorageBackend, StorageConfig};
    use bytes::Bytes;
    use std::path::Path;

    #[test]
    fn local_storage_paths_stay_inside_the_root() {
        let root = Path::new("storage-root");
        assert_eq!(
            safe_join(root, "bucket/extensions/file.crx").unwrap(),
            root.join("bucket/extensions/file.crx")
        );
        assert!(safe_join(root, "../outside").is_err());
        assert!(safe_join(root, "/absolute").is_err());
    }

    #[tokio::test]
    async fn local_storage_writes_objects_below_its_root() {
        let root = std::env::temp_dir().join(format!("simprint-storage-{}", uuid::Uuid::new_v4()));
        let config = StorageConfig {
            backend: StorageBackend::Local,
            endpoint: String::new(),
            public_base_url: String::new(),
            access_key: String::new(),
            secret_access_key: String::new(),
            bucket: "bucket".to_string(),
            avatar_root: "avatars".to_string(),
            extension_root: "extensions".to_string(),
            version_root: "versions".to_string(),
            local_path: Some(root.clone()),
        };
        let storage = ObjectStorage::new(&config).await.unwrap();

        storage
            .put_object(
                "bucket",
                "extensions/file.crx",
                Bytes::from_static(b"content"),
            )
            .await
            .unwrap();

        assert_eq!(
            tokio::fs::read(root.join("bucket/extensions/file.crx")).await.unwrap(),
            b"content"
        );
        tokio::fs::remove_dir_all(root).await.unwrap();
    }
}
