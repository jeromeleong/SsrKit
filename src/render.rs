use crate::island::IslandManager;
use crate::params::ParamsProcessor;
use crate::template::Template;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use regex::Regex;

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
        render_fn: F,
    ) -> Result<String, String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        let processed_params = self.params_processor.process(path, &params);

        let props = serde_json::json!({
            "url": path,
            "params": processed_params,
        });

        let content = render_fn(&props.to_string())?;
        let mut rendered = serde_json::from_str::<Value>(&content)
            .map_err(|e| format!("Failed to parse render result: {}", e))?;

        // 替换所有 Island 占位符并收集使用的 island IDs
        let mut used_islands = Vec::new();
        if let Some(html) = rendered["html"].as_str() {
            let (replaced_html, islands) = self.replace_island_placeholders(html)?;
            rendered["html"] = Value::String(replaced_html);
            used_islands = islands;
        }

        // 创建 islands 对象
        let islands_json = serde_json::json!({
            "islands": used_islands.into_iter().map(|id| (id, Value::Null)).collect::<serde_json::Map<String, Value>>()
        });

        self.template.render(&rendered, &islands_json)
    }

    fn replace_island_placeholders(&self, html: &str) -> Result<(String, Vec<String>), String> {
        let re = Regex::new(r#"<div data-island="([^"]+)"(?: data-props='([^']*)')?></div>"#).unwrap();
        let mut result = html.to_string();
        let mut used_islands = Vec::new();

        for cap in re.captures_iter(html) {
            let island_id = &cap[1];
            let props_str = cap.get(2).map_or("{}", |m| m.as_str());
            
            let props: Value = serde_json::from_str(props_str)
                .unwrap_or_else(|_| serde_json::json!({}));

            let rendered_island = self.island_manager.render_island(island_id, &props)?;
            result = result.replace(&cap[0], &rendered_island);
            used_islands.push(island_id.to_string());
        }

        Ok((result, used_islands))
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