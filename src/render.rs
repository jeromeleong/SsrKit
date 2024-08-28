use crate::init::RENDERER;
use crate::params::ParamsProcessor;
use crate::state::get_global_state;
use crate::template::Template;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "island")]
use crate::init::ISLAND_REGEX;
#[cfg(feature = "island")]
use crate::island::{IslandManager, IslandProcessor, ProcessContext};

pub struct SsrRenderer {
    params_processor: Box<dyn ParamsProcessor>,
    template: Arc<Template>,
    #[cfg(feature = "island")]
    island_manager: Arc<IslandManager>,
}

impl SsrRenderer {
    pub fn new(
        params_processor: Box<dyn ParamsProcessor>,
        #[cfg(feature = "island")] island_manager: Arc<IslandManager>,
        template: Arc<Template>,
    ) -> Self {
        Self {
            params_processor,
            template,
            #[cfg(feature = "island")]
            island_manager,
        }
    }

    pub fn render<F>(
        &self,
        path: &str,
        params: HashMap<String, String>,
        render_fn: F,
        #[cfg(feature = "island")] processor: &dyn IslandProcessor,
    ) -> Result<(String, Vec<String>), String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        let processed_params = self.params_processor.process(path, &params);

        let props = json!({
            "url": path,
            "params": processed_params,
        });

        let content = render_fn(&props.to_string())?;

        #[cfg(feature = "island")]
        {
            let mut rendered = serde_json::from_str::<Value>(&content)
                .map_err(|e| format!("Failed to parse render result: {}", e))?;
            // Conditional island processing
            if let Some(html) = rendered["html"].as_str() {
                if html.contains("data-island") {
                    let replaced_html = self.replace_island_placeholders(html)?;
                    rendered["html"] = Value::String(replaced_html);
                }
            }

            let context = ProcessContext {
                path: path.to_string(),
            };
            let islands_value = self.island_manager.process_islands(processor, &context);

            let global_state = get_global_state().read().map_err(|e| e.to_string())?;
            let cookie_manager = global_state
                .get_cookie_manager()
                .lock()
                .map_err(|e| e.to_string())?;
            let cookies = cookie_manager.to_header_strings();

            let html = self.template.render(&rendered, Some(&islands_value))?;

            Ok((html, cookies))
        }
        #[cfg(not(feature = "island"))]
        {
            let rendered = serde_json::from_str::<Value>(&content)
                .map_err(|e| format!("Failed to parse render result: {}", e))?;
            let global_state = get_global_state().read().map_err(|e| e.to_string())?;
            let cookie_manager = global_state
                .get_cookie_manager()
                .lock()
                .map_err(|e| e.to_string())?;
            let cookies = cookie_manager.to_header_strings();

            let html = self.template.render(&rendered)?;

            Ok((html, cookies))
        }
    }

    #[cfg(feature = "island")]
    fn replace_island_placeholders(&self, html: &str) -> Result<String, String> {
        let re = ISLAND_REGEX.get().expect("Regex not initialized");
        let mut result = html.to_string();

        for cap in re.captures_iter(html) {
            let island_id = &cap[1];
            let props_str = cap.get(2).map_or("{}", |m| m.as_str());

            let props: Value =
                serde_json::from_str(props_str).unwrap_or_else(|_| serde_json::json!({}));

            let rendered_island = self.island_manager.render_island(island_id, &props)?;
            result = result.replace(&cap[0], &rendered_island);
        }

        Ok(result)
    }

    #[cfg(feature = "island")]
    pub fn get_island_manager(&self) -> &Arc<IslandManager> {
        &self.island_manager
    }
}

pub fn get_renderer() -> &'static SsrRenderer {
    RENDERER.get().expect("Renderer not initialized")
}
