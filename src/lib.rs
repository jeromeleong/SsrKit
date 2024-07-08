use std::collections::HashMap;
use std::sync::OnceLock;
use serde_json::Value;

pub trait ParamsProcessor: Send + Sync + 'static {
    fn process(&self, path: &str, params: &HashMap<String, String>) -> serde_json::Map<String, Value>;
}

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

pub struct CombinedParamsProcessor {
    processors: Vec<(String, Box<dyn ParamsProcessor>)>,
}

impl CombinedParamsProcessor {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add<P: ParamsProcessor + 'static>(mut self, path_prefix: &str, processor: P) -> Self {
        self.processors.push((path_prefix.to_string(), Box::new(processor)));
        self
    }
}

impl ParamsProcessor for CombinedParamsProcessor {
    fn process(&self, path: &str, params: &HashMap<String, String>) -> serde_json::Map<String, Value> {
        for (prefix, processor) in &self.processors {
            if path.starts_with(prefix) {
                return processor.process(path, params);
            }
        }
        params.iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect()
    }
}

static RENDERER: OnceLock<SsrRenderer> = OnceLock::new();

pub fn get_or_init_renderer<P: ParamsProcessor>(init: impl FnOnce() -> P) -> &'static SsrRenderer {
    RENDERER.get_or_init(|| SsrRenderer::new(init()))
}