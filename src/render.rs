use std::collections::HashMap;
use std::sync::OnceLock;
use crate::params::ParamsProcessor;

pub struct SsrRenderer {
    params_processor: Box<dyn ParamsProcessor>,
}

impl SsrRenderer {
    pub fn new<P: ParamsProcessor>(params_processor: P) -> Self {
        Self {
            params_processor: Box::new(params_processor),
        }
    }

    pub fn render(&self, path: &str, params: HashMap<String, String>, render_fn: impl FnOnce(&str) -> Result<String, String>) -> Result<String, String> {
        let processed_params = self.params_processor.process(path, &params);
        let props = serde_json::json!({
            "url": path,
            "params": processed_params,
        });
        render_fn(&props.to_string())
    }
}

static RENDERER: OnceLock<SsrRenderer> = OnceLock::new();

pub fn get_or_init_renderer<P: ParamsProcessor>(init: impl FnOnce() -> P) -> &'static SsrRenderer {
    RENDERER.get_or_init(|| SsrRenderer::new(init()))
}