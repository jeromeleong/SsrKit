use ssrkit::prelude::*;
use std::collections::HashMap;

#[test]
fn test_combined_params_processor() {
    struct TestParamsProcessor;
    impl ParamsProcessor for TestParamsProcessor {
        fn process(
            &self,
            _path: &str,
            params: &HashMap<String, String>,
        ) -> serde_json::Map<String, Value> {
            params
                .iter()
                .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                .collect()
        }
    }

    let processor = CombinedParamsProcessor::new().add("/test", TestParamsProcessor);

    let params = HashMap::from([("key".to_string(), "value".to_string())]);
    let result = processor.process("/test", &params);

    assert_eq!(result.get("key"), Some(&Value::String("value".to_string())));
}
