use serde_json::Value;
use std::collections::HashMap;

pub trait ParamsProcessor: Send + Sync {
    fn process(
        &self,
        path: &str,
        params: &HashMap<String, String>,
    ) -> serde_json::Map<String, Value>;
}

pub struct CombinedParamsProcessor {
    processors: Vec<(String, Box<dyn ParamsProcessor>)>,
}

impl Default for CombinedParamsProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl CombinedParamsProcessor {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add<P: ParamsProcessor + 'static>(mut self, path_prefix: &str, processor: P) -> Self {
        self.processors
            .push((path_prefix.to_string(), Box::new(processor)));
        self
    }
}

impl ParamsProcessor for CombinedParamsProcessor {
    fn process(
        &self,
        path: &str,
        params: &HashMap<String, String>,
    ) -> serde_json::Map<String, Value> {
        for (prefix, processor) in &self.processors {
            if path.starts_with(prefix) {
                return processor.process(path, params);
            }
        }
        params
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect()
    }
}
