use crate::island::IslandManager;
use crate::params::ParamsProcessor;
use crate::template::Template;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

pub struct SsrRenderer {
    params_processor: Box<dyn ParamsProcessor>,
    island_manager: Arc<IslandManager>,
    template: Arc<Template>,
}

impl SsrRenderer {
    pub fn new<P: ParamsProcessor + 'static>(
        params_processor: P,
        island_manager: Arc<IslandManager>,
        template: Arc<Template>,
    ) -> Self {
        Self {
            params_processor: Box::new(params_processor),
            island_manager,
            template,
        }
    }

    pub fn render<F>(
        &self,
        path: &str,
        params: HashMap<String, String>,
        island_ids: Option<Vec<&str>>,
        render_fn: F,
    ) -> Result<String, String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        let processed_params = self.params_processor.process(path, &params);

        let islands_json = if let Some(ids) = island_ids {
            let mut island_map = serde_json::Map::new();
            for id in ids {
                match self.island_manager.render_island(id, &Value::Null) {
                    Ok(rendered) => {
                        island_map.insert(id.to_string(), Value::String(rendered));
                    }
                    Err(e) => {
                        return Err(format!("Failed to render island '{}': {}", id, e));
                    }
                }
            }
            Value::Object(island_map)
        } else {
            Value::Object(serde_json::Map::new())
        };

        let props = serde_json::json!({
            "url": path,
            "params": processed_params,
            "islands": islands_json,
        });

        let content = render_fn(&props.to_string())?;
        let rendered = serde_json::from_str::<Value>(&content)
            .map_err(|e| format!("Failed to parse render result: {}", e))?;

        self.template.render(&rendered, &islands_json)
    }
}

pub fn init_renderer<P: ParamsProcessor + 'static>(
    init: impl FnOnce() -> P,
    island_manager: Arc<IslandManager>,
    template: Arc<Template>,
) -> &'static SsrRenderer {
    static RENDERER: OnceLock<SsrRenderer> = OnceLock::new();
    RENDERER.get_or_init(|| SsrRenderer::new(init(), island_manager, template))
}
