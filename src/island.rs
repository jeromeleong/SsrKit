use crate::config::get_global_config;
use crate::{Cache, SsrkitConfig};
use nanoid::nanoid;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

pub type IslandRenderer =
    dyn for<'a> Fn(&'a str, &'a Value) -> Result<String, String> + Send + Sync;

static ISLAND_CACHE: OnceLock<Cache<String>> = OnceLock::new();

pub struct Island {
    pub id: Cow<'static, str>,
    pub version: u64,
    pub meta: Option<Value>,
}

pub struct IslandManager {
    islands: Arc<Mutex<HashMap<Cow<'static, str>, Island>>>,
    renderers: Arc<Mutex<HashMap<Cow<'static, str>, Arc<IslandRenderer>>>>,
    config: Arc<SsrkitConfig>,
}

impl Default for IslandManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandManager {
    pub fn new() -> Self {
        let config = get_global_config().clone();
        Self {
            islands: Arc::new(Mutex::new(HashMap::new())),
            renderers: Arc::new(Mutex::new(HashMap::new())),
            config: Arc::new(config),
        }
    }

    pub fn register(&self) -> IslandRegistration {
        IslandRegistration::new(self)
    }

    pub fn add_island(
        &self,
        id: impl Into<Cow<'static, str>>,
        default_props: Option<Value>,
    ) -> Result<(), String> {
        let mut islands = self.islands.lock().map_err(|e| e.to_string())?;
        let id = id.into();
        let version = islands.get(&id).map_or(1, |island| island.version + 1);
        islands.insert(
            id.clone(),
            Island {
                id,
                version,
                meta: default_props,
            },
        );
        Ok(())
    }

    pub fn render_island(&self, id: &str, instance_props: &Value) -> Result<String, String> {
        let islands = self.islands.lock().map_err(|e| e.to_string())?;
        let renderers = self.renderers.lock().unwrap();
        let island = islands
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
        (renderer)(id, &merged_props)
    }

    pub fn get_manifest_json(&self) -> Result<Value, String> {
        self.islands.lock().map_err(|e| e.to_string()).map(|guard| {
            Value::Object(
                guard
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
        })
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
            islands: self.islands.clone(),
            renderers: self.renderers.clone(),
            config: self.config.clone(),
        }
    }
}

pub fn init_island_cache() {
    ISLAND_CACHE.get_or_init(|| Cache::new(|config| config.get_island_cache_size()));
}

pub fn get_or_render_island<F>(key: &str, render_fn: F) -> String
where
    F: FnOnce() -> String,
{
    ISLAND_CACHE
        .get()
        .expect("Island cache not initialized")
        .get_or_insert(key, render_fn)
}

pub struct ProcessContext {
    pub path: String,
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

    #[allow(clippy::should_implement_trait)]
    pub fn add<P: IslandProcessor + 'static>(mut self, processor: P) -> Self {
        self.processors.push(Box::new(processor));
        self
    }
}

impl Default for CombinedIslandProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl IslandProcessor for CombinedIslandProcessor {
    fn process(&self, island_manager: &Arc<IslandManager>, context: &ProcessContext) -> Value {
        let mut result = serde_json::Map::new();

        for processor in &self.processors {
            let processed_value = processor.process(island_manager, context);
            if let Some(obj) = processed_value.as_object() {
                for (key, value) in obj {
                    result.insert(key.clone(), value.clone());
                }
            }
        }

        Value::Object(result)
    }
}

pub struct IslandRegistration<'a> {
    manager: &'a IslandManager,
}

impl<'a> IslandRegistration<'a> {
    pub fn new(manager: &'a IslandManager) -> Self {
        Self { manager }
    }

    pub fn add_id(self, id: impl Into<Cow<'static, str>>) -> Self {
        let id = id.into();
        self.manager
            .renderers
            .lock()
            .unwrap()
            .insert(id.clone(), Arc::new(default_renderer));
        let _ = self.manager.add_island(id, None);
        self
    }

    pub fn add<F>(
        self,
        id: impl Into<Cow<'static, str>>,
        renderer: F,
        default_props: Option<Value>,
    ) -> Self
    where
        F: for<'b> Fn(&'b str, &'b Value) -> Result<String, String> + Send + Sync + 'static,
    {
        let id = id.into();
        self.manager
            .renderers
            .lock()
            .unwrap()
            .insert(id.clone(), Arc::new(renderer));
        let _ = self.manager.add_island(id, default_props);
        self
    }

    pub fn finish(self) -> IslandManager {
        self.manager.clone()
    }
}

fn default_renderer(id: &str, props: &Value) -> Result<String, String> {
    let name = props["name"].as_str().unwrap_or("default");
    let client_strategy = props["client"].as_str().unwrap_or("load");
    let instance_id = props["instanceId"].as_str().unwrap_or("");

    Ok(indoc::formatdoc! {r#"
        <div data-island="{id}" data-name="{name}" data-instance-id="{instance_id}" data-client="{client_strategy}">
        </div>
    "#})
}
