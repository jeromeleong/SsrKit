use nanoid::nanoid;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub type IslandRenderer = dyn Fn(&str, &Value) -> Result<String, String> + Send + Sync;

pub struct Island {
    pub id: Cow<'static, str>,
    pub version: u64,
    pub meta: Option<Value>,
}

pub struct IslandManifest {
    islands: HashMap<Cow<'static, str>, Island>,
}

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

pub struct ProcessContext {
    pub path: String,
    // 可以根据需要添加更多字段
}

pub trait IslandProcessor: Send + Sync {
    fn process(&self, island_manager: &Arc<IslandManager>, context: &ProcessContext) -> Value;
}

pub struct CombinedIslandProcessor {
    processors: Vec<Box<dyn IslandProcessor>>,
}

impl CombinedIslandProcessor {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add<P: IslandProcessor + 'static>(mut self, processor: P) -> Self {
        self.processors.push(Box::new(processor));
        self
    }
}

impl IslandProcessor for CombinedIslandProcessor {
    fn process(&self, island_manager: &Arc<IslandManager>, context: &ProcessContext) -> Value {
        let mut result = serde_json::Map::new();
        for processor in &self.processors {
            let processed = processor.process(island_manager, context);
            if let Value::Object(map) = processed {
                result.extend(map);
            }
        }
        Value::Object(result)
    }
}

pub struct IslandManager {
    manifest: Arc<Mutex<IslandManifest>>,
    renderers: Arc<Mutex<HashMap<Cow<'static, str>, Box<IslandRenderer>>>>,
}

impl IslandManager {
    pub fn new() -> Self {
        Self {
            manifest: Arc::new(Mutex::new(IslandManifest::new())),
            renderers: Arc::new(Mutex::new(HashMap::new())),
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
        let instance_id = nanoid!(10);
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

impl Clone for IslandManager {
    fn clone(&self) -> Self {
        Self {
            manifest: self.manifest.clone(),
            renderers: self.renderers.clone(),
        }
    }
}

static ISLAND_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

pub fn get_or_render_island<F>(key: &str, render_fn: F) -> String
where
    F: FnOnce() -> String,
{
    let cache = ISLAND_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut cache_guard = cache.lock().unwrap();
    match cache_guard.get(key) {
        Some(cached) => cached.clone(),
        None => {
            let rendered = render_fn();
            cache_guard.insert(key.to_string(), rendered.clone());
            rendered
        }
    }
}