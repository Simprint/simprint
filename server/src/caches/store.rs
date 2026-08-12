use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use dashmap::{DashMap, mapref::entry::Entry};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct CacheStore(Arc<MemoryCache>);

#[derive(Default)]
pub struct MemoryCache {
    entries: DashMap<String, CacheEntry>,
    operations: AtomicUsize,
}

struct CacheEntry {
    value: String,
    expires_at: Option<Instant>,
}

impl CacheEntry {
    fn is_expired(&self, now: Instant) -> bool {
        self.expires_at.is_some_and(|expires_at| expires_at <= now)
    }
}

impl CacheStore {
    pub fn memory() -> Self {
        Self(Arc::new(MemoryCache::default()))
    }

    pub async fn get_string(&self, key: &str) -> Result<Option<String>, anyhow::Error> {
        Ok(self.0.get(key))
    }

    pub async fn set_string(
        &self,
        key: impl Into<String>,
        value: impl Into<String>,
        ttl: Duration,
    ) -> Result<(), anyhow::Error> {
        let key = key.into();
        let value = value.into();
        self.0.set(key, value, ttl);
        Ok(())
    }

    pub async fn get_json<T>(&self, key: &str) -> Result<Option<T>, anyhow::Error>
    where
        T: DeserializeOwned,
    {
        self.get_string(key)
            .await?
            .map(|value| serde_json::from_str(&value))
            .transpose()
            .map_err(Into::into)
    }

    pub async fn set_json<T>(
        &self,
        key: impl Into<String>,
        value: &T,
        ttl: Duration,
    ) -> Result<(), anyhow::Error>
    where
        T: Serialize,
    {
        self.set_string(key, serde_json::to_string(value)?, ttl).await
    }

    pub async fn delete(&self, key: &str) -> Result<(), anyhow::Error> {
        self.0.entries.remove(key);
        Ok(())
    }

    pub async fn delete_prefix(&self, prefix: &str) -> Result<(), anyhow::Error> {
        self.0.delete_prefix(prefix);
        Ok(())
    }

    pub async fn get_i64(&self, key: &str) -> Result<Option<i64>, anyhow::Error> {
        self.get_string(key)
            .await?
            .map(|value| value.parse::<i64>())
            .transpose()
            .map_err(Into::into)
    }

    pub async fn increment(&self, key: &str, ttl: Duration) -> Result<i64, anyhow::Error> {
        Ok(self.0.increment(key, ttl)?)
    }
}

impl MemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        self.maybe_prune();
        let now = Instant::now();
        let value = self.entries.get(key).and_then(|entry| {
            if entry.is_expired(now) {
                None
            } else {
                Some(entry.value.clone())
            }
        });

        if value.is_none() {
            self.entries.remove_if(key, |_, entry| entry.is_expired(now));
        }
        value
    }

    fn set(&self, key: String, value: String, ttl: Duration) {
        self.maybe_prune();
        self.entries.insert(
            key,
            CacheEntry {
                value,
                expires_at: Some(Instant::now() + ttl),
            },
        );
    }

    fn delete_prefix(&self, prefix: &str) {
        let keys = self
            .entries
            .iter()
            .filter(|entry| entry.key().starts_with(prefix))
            .map(|entry| entry.key().clone())
            .collect::<Vec<_>>();
        for key in keys {
            self.entries.remove(&key);
        }
    }

    fn increment(&self, key: &str, ttl: Duration) -> Result<i64, std::num::ParseIntError> {
        self.maybe_prune();
        let now = Instant::now();
        match self.entries.entry(key.to_string()) {
            Entry::Occupied(mut entry) if !entry.get().is_expired(now) => {
                let count = entry.get().value.parse::<i64>()? + 1;
                entry.get_mut().value = count.to_string();
                Ok(count)
            }
            Entry::Occupied(mut entry) => {
                entry.insert(CacheEntry {
                    value: "1".to_string(),
                    expires_at: Some(now + ttl),
                });
                Ok(1)
            }
            Entry::Vacant(entry) => {
                entry.insert(CacheEntry {
                    value: "1".to_string(),
                    expires_at: Some(now + ttl),
                });
                Ok(1)
            }
        }
    }

    fn maybe_prune(&self) {
        const PRUNE_INTERVAL: usize = 256;
        if self.operations.fetch_add(1, Ordering::Relaxed) % PRUNE_INTERVAL != 0 {
            return;
        }

        let now = Instant::now();
        self.entries.retain(|_, entry| !entry.is_expired(now));
    }
}

#[cfg(test)]
mod tests {
    use super::CacheStore;
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Value {
        name: String,
    }

    #[tokio::test]
    async fn memory_cache_supports_values_prefixes_and_counters() {
        let cache = CacheStore::memory();
        cache
            .set_json(
                "group:one",
                &Value { name: "one".into() },
                Duration::from_secs(30),
            )
            .await
            .unwrap();
        cache.set_string("group:two", "two", Duration::from_secs(30)).await.unwrap();

        assert_eq!(
            cache.get_json::<Value>("group:one").await.unwrap(),
            Some(Value { name: "one".into() })
        );
        assert_eq!(
            cache.increment("counter", Duration::from_secs(30)).await.unwrap(),
            1
        );
        assert_eq!(
            cache.increment("counter", Duration::from_secs(30)).await.unwrap(),
            2
        );

        cache.delete_prefix("group:").await.unwrap();
        assert_eq!(cache.get_string("group:one").await.unwrap(), None);
        assert_eq!(cache.get_string("group:two").await.unwrap(), None);

        cache.set_string("expired", "value", Duration::ZERO).await.unwrap();
        assert_eq!(cache.get_string("expired").await.unwrap(), None);
    }
}
