use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Island {
    pub id: String,
    pub content: String,
    pub version: u64,
    pub meta: Option<serde_json::Value>,
}

pub struct IslandManifest {
    islands: HashMap<String, Island>,
}

impl IslandManifest {
    pub fn new() -> Self {
        Self {
            islands: HashMap::new(),
        }
    }

    pub fn add(&mut self, id: impl Into<String>, content: impl Into<String>) -> &mut Island {
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

    pub fn update(&mut self, id: &str, content: impl Into<String>) -> Option<&mut Island> {
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

    pub fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        for (id, island) in &self.islands {
            map.insert(
                id.clone(),
                serde_json::json!({
                    "id": island.id,
                    "content": island.content,
                    "version": island.version,
                    "meta": island.meta,
                }),
            );
        }
        serde_json::Value::Object(map)
    }
}

// Addressing the new_without_default warning
impl Default for IslandManifest {
    fn default() -> Self {
        Self::new()
    }
}

type IslandRendererFn = dyn Fn(&str, &Value) -> Result<String, String> + Send + Sync;

pub struct IslandManager {
    manifest: Arc<Mutex<IslandManifest>>,
    renderer: Arc<IslandRendererFn>,
}

impl IslandManager {
    pub fn new(
        renderer: impl Fn(&str, &Value) -> Result<String, String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            manifest: Arc::new(Mutex::new(IslandManifest::new())),
            renderer: Arc::new(renderer),
        }
    }

    pub fn add_island(
        &self,
        id: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<(), String> {
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        manifest.add(id, content);
        Ok(())
    }

    pub fn render_island(&self, id: &str, props: &serde_json::Value) -> Result<String, String> {
        let manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        let island = manifest
            .get(id)
            .ok_or_else(|| format!("Island '{}' not found", id))?;

        let mut island_props = props.clone();
        if let Some(obj) = island_props.as_object_mut() {
            obj.insert(
                "islandId".to_string(),
                serde_json::Value::String(id.to_string()),
            );
            obj.insert(
                "version".to_string(),
                serde_json::Value::Number(island.version.into()),
            );
        }

        (self.renderer)(id, &island_props)
    }

    pub fn update_island(&self, id: &str, content: impl Into<String>) -> Result<(), String> {
        let mut manifest = self.manifest.lock().map_err(|e| e.to_string())?;
        manifest
            .update(id, content)
            .ok_or_else(|| format!("Island '{}' not found", id))?;
        Ok(())
    }

    pub fn get_manifest_json(&self) -> Result<serde_json::Value, String> {
        self.manifest
            .lock()
            .map_err(|e| e.to_string())
            .map(|guard| guard.to_json())
    }
}
