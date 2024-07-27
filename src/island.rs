use nanoid::nanoid;
use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type IslandRenderer = dyn Fn(&str, &Value) -> Result<String, String> + Send + Sync;

pub struct Island {
    pub id: Cow<'static, str>,
    pub version: u64,
    pub meta: Option<Value>, // 用于存储默认属性
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

impl Default for IslandManifest {
    fn default() -> Self {
        Self::new()
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
    
        let instance_id = nanoid!(10); // 生成10位的nanoid
        
        // 合并默认属性和实例特定属性
        let mut merged_props = serde_json::json!({
            "islandId": id,
            "version": island.version,
            "instanceId": instance_id
        });
        if let Some(obj) = merged_props.as_object_mut() {
            // 添加岛屿的默认属性
            if let Some(default_props) = &island.meta {
                for (key, value) in default_props.as_object().unwrap() {
                    obj.insert(key.clone(), value.clone());
                }
            }
            // 用实例特定属性覆盖
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
}

impl Default for IslandManager {
    fn default() -> Self {
        Self::new()
    }
}