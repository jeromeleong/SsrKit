use crate::config::SsrkitConfig;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Mutex, OnceLock};

static CONFIG: OnceLock<SsrkitConfig> = OnceLock::new();

pub struct Cache<T> {
    cache: OnceLock<Mutex<LruCache<String, T>>>,
    cache_size_fn: Box<dyn Fn(&SsrkitConfig) -> NonZeroUsize + Send + Sync>,
}

impl<T: Clone> Cache<T> {
    pub fn new(
        cache_size_fn: impl Fn(&SsrkitConfig) -> NonZeroUsize + Send + Sync + 'static,
    ) -> Self {
        Self {
            cache: OnceLock::new(),
            cache_size_fn: Box::new(cache_size_fn),
        }
    }

    fn get_or_create_cache(&self) -> &Mutex<LruCache<String, T>> {
        self.cache.get_or_init(|| {
            let config = CONFIG.get().cloned().unwrap_or_else(SsrkitConfig::default);
            Mutex::new(LruCache::new((self.cache_size_fn)(&config)))
        })
    }

    pub fn insert(&self, key: &str, value: T) -> T {
        let mut cache_guard = self.get_or_create_cache().lock().unwrap();
        cache_guard.put(key.to_string(), value.clone());
        value
    }

    pub fn get(&self, key: &str) -> Option<T> {
        let mut cache_guard = self.get_or_create_cache().lock().unwrap();
        cache_guard.get(key).cloned()
    }

    pub fn get_or_insert<F>(&self, key: &str, create_fn: F) -> T
    where
        F: FnOnce() -> T,
    {
        if let Some(value) = self.get(key) {
            value
        } else {
            let new_value = create_fn();
            self.insert(key, new_value)
        }
    }
}

pub fn init_cache(config: &SsrkitConfig) {
    let _ = CONFIG.set(config.clone());
}
