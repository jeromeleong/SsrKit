#[cfg(feature = "island")]
use crate::config::get_global_config;
#[cfg(feature = "island")]
use crate::{Cache, SsrkitConfig};
#[cfg(feature = "island")]
use nanoid::nanoid;
#[cfg(feature = "island")]
use serde_json::Value;
#[cfg(feature = "island")]
use std::borrow::Cow;
#[cfg(feature = "island")]
use std::collections::HashMap;
#[cfg(feature = "island")]
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(feature = "island")]
pub type IslandRenderer = dyn Fn(&str, &Value) -> Result<String, String> + Send + Sync;

#[cfg(feature = "island")]
static ISLAND_CACHE: OnceLock<Cache<String>> = OnceLock::new();

#[cfg(feature = "island")]
pub struct Island {
    pub id: Cow<'static, str>,
    pub version: u64,
    pub meta: Option<Value>,
}

#[cfg(feature = "island")]
pub struct IslandManifest {
    islands: HashMap<Cow<'static, str>, Island>,
}

#[cfg(feature = "island")]
impl Default for IslandManifest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "island")]
impl IslandManifest {
    pub fn new() -> Self {
        Self {
            islands: HashMap::new(),
        }
    }

    pub fn add(
        &mut self,
        id: impl Into<Cow<'static, str>>,
        default_props: Option<Value>,
    ) -> &mut Island {
        let id = id.into();
        let version = self.islands.get(&id).map_or(1, |island| island.version + 1);
        self.islands.entry(id.clone()).or_insert_with(|| Island {
            id,
            version,
            meta: default_props,
        })
    }

    pub fn get(&self, id: &str) -> Option<&Island> {
        self.islands.get(id)
    }

    pub fn to_json(&self) -> Value {
        Value::Object(
            self.islands
                .iter()
                .map(|(id, island)| {
                    (
                        id.to_string(),
                        serde_json::json!({
                            "id": island.id,
                            "version": island.version,
                            "meta": island.meta,
                        }),
                    )
                })
                .collect(),
        )
    }
}

#[cfg(feature = "island")]
pub struct ProcessContext {
    pub path: String,
}

#[cfg(feature = "island")]
pub trait IslandProcessor: Send + Sync {
    fn process(&self, island_manager: &Arc<IslandManager>, context: &ProcessContext) -> Value;
}

#[cfg(feature = "island")]
pub struct IslandManager {
    manifest: Arc<Mutex<IslandManifest>>,
    renderers: Arc<Mutex<HashMap<Cow<'static, str>, Box<IslandRenderer>>>>,
    config: Arc<SsrkitConfig>,
}

#[cfg(feature = "island")]
impl Default for IslandManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "island")]
impl IslandManager {
    pub fn new() -> Self {
        let config = get_global_config().clone();
        Self {
            manifest: Arc::new(Mutex::new(IslandManifest::new())),
            renderers: Arc::new(Mutex::new(HashMap::new())),
            config: config.into(),
        }
    }

    pub fn register<F>(&self, id: impl Into<Cow<'static, str>>, renderer: F)
    where
        F: Fn(&str, &Value) -> Result<String, String> + Send + Sync + 'static,
    {
        let mut renderers = self.renderers.lock().unwrap();
        renderers.insert(id.into(), Box::new(renderer));
    }

    pub fn add_island(
        &self,
        id: impl Into<Cow<'static, str>>,
        default_props: Option<Value>,
    ) -> Result<(), String> {
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        manifest.add(id, default_props);
        Ok(())
    }

    pub fn render_island(&self, id: &str, instance_props: &Value) -> Result<String, String> {
        let manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let renderers = self.renderers.lock().unwrap();
        let island = manifest
            .get(id)
            .ok_or_else(|| format!("Island '{}' not found in manifest", id))?;
        let renderer = renderers
            .get(id)
            .ok_or_else(|| format!("Renderer for island '{}' not found", id))?;
        let length = self.config.get_nanoid_length();
        let alphabet = self.config.get_nanoid_alphabet();
        let instance_id = nanoid!(length, &alphabet);
        let mut merged_props = serde_json::json!({
            "islandId": id,
            "version": island.version,
            "instanceId": instance_id
        });
        if let Some(obj) = merged_props.as_object_mut() {
            if let Some(default_props) = &island.meta {
                for (key, value) in default_props.as_object().unwrap() {
                    obj.insert(key.clone(), value.clone());
                }
            }
            for (key, value) in instance_props.as_object().unwrap() {
                obj.insert(key.clone(), value.clone());
            }
        }
        renderer(id, &merged_props)
    }

    pub fn get_manifest_json(&self) -> Result<Value, String> {
        self.manifest
            .lock()
            .map_err(|e| e.to_string())
            .map(|guard| guard.to_json())
    }

    pub fn process_islands(
        &self,
        processor: &dyn IslandProcessor,
        context: &ProcessContext,
    ) -> Value {
        processor.process(&Arc::new(self.clone()), context)
    }
}

#[cfg(feature = "island")]
impl Clone for IslandManager {
    fn clone(&self) -> Self {
        Self {
            manifest: self.manifest.clone(),
            renderers: self.renderers.clone(),
            config: self.config.clone(),
        }
    }
}

#[cfg(feature = "island")]
pub fn init_island_cache() {
    ISLAND_CACHE.get_or_init(|| Cache::new(|config| config.get_island_cache_size()));
}

#[cfg(feature = "island")]
pub fn get_or_render_island<F>(key: &str, render_fn: F) -> String
where
    F: FnOnce() -> String,
{
    ISLAND_CACHE
        .get()
        .expect("Island cache not initialized")
        .get_or_insert(key, render_fn)
}
