use crate::island::IslandManager;
use crate::params::ParamsProcessor;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

pub type IslandRenderer =
    Box<dyn Fn(&str, &serde_json::Value) -> Result<String, String> + Send + Sync>;

pub struct SsrRenderer {
    params_processor: Box<dyn ParamsProcessor>,
    island_manager: Option<IslandManager>,
}

impl SsrRenderer {
    pub fn new<P: ParamsProcessor>(
        params_processor: P,
        island_renderer: Option<IslandRenderer>,
    ) -> Self {
        Self {
            params_processor: Box::new(params_processor),
            island_manager: island_renderer.map(IslandManager::new),
        }
    }

    pub fn render(
        &self,
        path: &str,
        params: HashMap<String, String>,
        island_ids: Option<Vec<String>>,
        render_fn: impl FnOnce(&str) -> Result<String, String>,
    ) -> Result<Value, String> {
        let processed_params = self.params_processor.process(path, &params);

        let islands_json = if let (Some(manager), Some(ids)) = (&self.island_manager, island_ids) {
            for id in &ids {
                let island_props = serde_json::json!({
                    "url": path,
                    "params": processed_params,
                    "islandId": id,
                });
                let island_content = manager.render_island(id, &island_props)?;
                manager.add_island(id, island_content)?;
            }
            manager.get_manifest_json()?
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        let props = serde_json::json!({
            "url": path,
            "params": processed_params,
            "islands": islands_json,
        });

        let result = render_fn(&props.to_string())?;
        let mut parsed_result: Value = serde_json::from_str(&result)
            .map_err(|e| format!("Failed to parse render result: {}", e))?;

        parsed_result["islands"] = islands_json;

        Ok(parsed_result)
    }
}

static RENDERER: OnceLock<SsrRenderer> = OnceLock::new();

pub fn init_renderer<P: ParamsProcessor>(
    init: impl FnOnce() -> P,
    island_renderer: Option<IslandRenderer>,
) -> &'static SsrRenderer {
    RENDERER.get_or_init(|| SsrRenderer::new(init(), island_renderer))
}
