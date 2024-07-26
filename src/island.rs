use serde_json::Value;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type IslandRenderer = dyn Fn(&str, &Value) -> Result<String, String> + Send + Sync;

pub struct Island {
    pub id: Cow<'static, str>,
    pub content: Cow<'static, str>,
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
        content: impl Into<Cow<'static, str>>,
    ) -> &mut Island {
        let id = id.into();
        let content = content.into();
        let version = self.islands.get(&id).map_or(1, |island| island.version + 1);
        self.islands.entry(id.clone()).or_insert_with(|| Island {
            id,
            content,
            version,
            meta: None,
        })
    }

    pub fn get(&self, id: &str) -> Option<&Island> {
        self.islands.get(id)
    }

    pub fn update(
        &mut self,
        id: &str,
        content: impl Into<Cow<'static, str>>,
    ) -> Option<&mut Island> {
        self.islands.get_mut(id).map(|island| {
            island.content = content.into();
            island.version += 1;
            island
        })
    }

    pub fn remove(&mut self, id: &str) -> Option<Island> {
        self.islands.remove(id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Island> {
        self.islands.values()
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
                            "content": island.content,
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
        content: impl Into<Cow<'static, str>>,
    ) -> Result<(), String> {
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        manifest.add(id, content);
        Ok(())
    }

    pub fn render_island(&self, id: &str, props: &Value) -> Result<String, String> {
        let manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let renderers = self.renderers.lock().unwrap();

        let island = manifest
            .get(id)
            .ok_or_else(|| format!("Island '{}' not found in manifest", id))?;
        let renderer = renderers
            .get(id)
            .ok_or_else(|| format!("Renderer for island '{}' not found", id))?;

        let mut island_props = props.clone();
        if let Some(obj) = island_props.as_object_mut() {
            obj.insert("islandId".to_string(), Value::String(id.to_string()));
            obj.insert("version".to_string(), Value::Number(island.version.into()));
        }

        renderer(id, &island_props)
    }

    pub fn update_island(
        &self,
        id: &str,
        content: impl Into<Cow<'static, str>>,
    ) -> Result<(), String> {
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        manifest
            .update(id, content)
            .ok_or_else(|| format!("Island '{}' not found", id))?;
        Ok(())
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
