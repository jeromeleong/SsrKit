use crate::island::{init_island_cache, IslandManager, IslandProcessor, ProcessContext};
use crate::params::ParamsProcessor;
use crate::template::{init_template_cache, Template};
use crate::SsrkitConfig;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;

// 全局靜態變量
static ISLAND_REGEX: OnceLock<Regex> = OnceLock::new();
static RENDERER: OnceLock<SsrRenderer> = OnceLock::new();
static ISLAND_MANAGER: OnceLock<Arc<IslandManager>> = OnceLock::new();
static TEMPLATE: OnceLock<Arc<Template>> = OnceLock::new();

pub struct SsrRenderer {
    params_processor: Box<dyn ParamsProcessor>,
    island_manager: Arc<IslandManager>,
    template: Arc<Template>,
}

impl SsrRenderer {
    fn new(
        params_processor: Box<dyn ParamsProcessor>,
        island_manager: Arc<IslandManager>,
        template: Arc<Template>,
    ) -> Self {
        Self {
            params_processor,
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

        // 條件性島嶼處理
        let mut used_islands = Vec::new();
        if let Some(html) = rendered["html"].as_str() {
            if html.contains("data-island") {
                let (replaced_html, islands) = self.replace_island_placeholders(html)?;
                rendered["html"] = Value::String(replaced_html);
                used_islands = islands;
            } else {
            }
        }

        // 創建 islands 對象
        let islands_json = if !used_islands.is_empty() {
            serde_json::json!({
                "islands": used_islands.into_iter().map(|id| (id, Value::Null)).collect::<serde_json::Map<String, Value>>()
            })
        } else {
            serde_json::json!({})
        };

        let result = self.template.render(&rendered, &islands_json);
        result
    }

    fn replace_island_placeholders(&self, html: &str) -> Result<(String, Vec<String>), String> {
        let re = ISLAND_REGEX.get().expect("Regex not initialized");
        let mut result = html.to_string();
        let mut used_islands = Vec::new();

        for cap in re.captures_iter(html) {
            let island_id = &cap[1];
            let props_str = cap.get(2).map_or("{}", |m| m.as_str());

            let props: Value =
                serde_json::from_str(props_str).unwrap_or_else(|_| serde_json::json!({}));

            let rendered_island = self.island_manager.render_island(island_id, &props)?;
            result = result.replace(&cap[0], &rendered_island);
            used_islands.push(island_id.to_string());
        }

        Ok((result, used_islands))
    }

    pub fn get_island_manager(&self) -> &Arc<IslandManager> {
        &self.island_manager
    }

    pub fn process_and_render<P, F>(
        &self,
        processor: &P,
        path: &str,
        params: HashMap<String, String>,
        render_fn: F
    ) -> Result<String, String>
    where
        P: IslandProcessor,
        F: FnOnce(&str) -> Result<String, String>,
    {
        let context = ProcessContext { path: path.to_string() };
        let islands_value = self.island_manager.process_islands(processor, &context);
        
        let content = self.render(path, params, render_fn)?;
        
        // 尝试解析 content 为 JSON，如果失败则将其作为字符串处理
        let content_value = serde_json::from_str::<Value>(&content).unwrap_or_else(|_| {
            json!({ "html": content })
        });
    
        self.template.render(&content_value, &islands_value)
    }
}

pub fn init_ssr(
    params_processor_init: impl FnOnce() -> Box<dyn ParamsProcessor>,
    island_manager_init: impl FnOnce() -> IslandManager,
    template_init: impl FnOnce() -> Template,
    config: Option<&SsrkitConfig>,
) {
    let config = config.cloned().unwrap_or_default();

    // 初始化正則表達式
    ISLAND_REGEX.get_or_init(|| {
        Regex::new(r#"<div data-island="([^"]+)"(?: data-props='([^']*)')?></div>"#).unwrap()
    });

    // 初始化 IslandManager
    let island_manager = island_manager_init();
    init_island_cache(&config);
    ISLAND_MANAGER.get_or_init(|| Arc::new(island_manager));

    // 初始化 Template
    let template = template_init();
    init_template_cache(&config);
    TEMPLATE.get_or_init(|| Arc::new(template));

    // 初始化 Renderer
    RENDERER.get_or_init(|| {
        SsrRenderer::new(
            params_processor_init(),
            ISLAND_MANAGER.get().unwrap().clone(),
            TEMPLATE.get().unwrap().clone(),
        )
    });

}

pub fn get_renderer() -> &'static SsrRenderer {
    RENDERER.get().expect("Renderer not initialized")
}