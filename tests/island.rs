#[cfg(feature = "island")]
use serde_json::json;
#[cfg(feature = "island")]
use ssrkit::prelude::*;
#[cfg(feature = "island")]
use std::sync::Arc;

#[cfg(feature = "island")]
#[test]
fn test_island_registration_and_rendering() {
    // 測試 island 的註冊和渲染
    set_global_config(SsrkitConfig::default());

    let manager = IslandManager::new();

    let registration = manager.register();
    let id = "test_island";
    let renderer = |_: &str, _: &serde_json::Value| Ok("test island content".to_string());
    let default_props = Some(json!({ "prop": "value" }));

    registration.add(id, renderer, default_props);

    let result = manager.render_island(id, &json!({}));
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("test island content"));
}

#[cfg(feature = "island")]
#[test]
fn test_island_manifest() {
    // 測試 island 清單
    set_global_config(SsrkitConfig::default());

    let manager = IslandManager::new();

    let registration = manager.register();
    let id = "test_island";
    let renderer = |_: &str, _: &serde_json::Value| Ok("test island content".to_string());
    let default_props = Some(json!({ "prop": "value" }));

    registration.add(id, renderer, default_props);

    let manifest = manager.get_manifest_json();
    assert!(manifest.is_ok());
    let manifest = manifest.unwrap();

    assert!(manifest.as_object().unwrap().contains_key("test_island"));
}

#[cfg(feature = "island")]
#[test]
fn test_island_processor() {
    // 測試 island 處理器
    set_global_config(SsrkitConfig::default());

    let manager = IslandManager::new();

    let registration = manager.register();
    let id = "test_island";
    let renderer = |_: &str, _: &serde_json::Value| Ok("test island content".to_string());
    let default_props = Some(json!({ "prop": "value" }));

    registration.add(id, renderer, default_props);

    struct TestProcessor;
    impl IslandProcessor for TestProcessor {
        fn process(
            &self,
            island_manager: &Arc<IslandManager>,
            context: &ProcessContext,
        ) -> serde_json::Value {
            let mut result = serde_json::Map::new();
            result.insert(
                "path".to_string(),
                serde_json::Value::String(context.path.clone()),
            );
            result.insert(
                "islands".to_string(),
                island_manager.get_manifest_json().unwrap(),
            );
            serde_json::Value::Object(result)
        }
    }

    let processor = TestProcessor;
    let context = ProcessContext {
        path: "/test".to_string(),
    };

    let processed_value = manager.process_islands(&processor, &context);
    assert!(processed_value.is_object());
    let processed_obj = processed_value.as_object().unwrap();

    assert!(processed_obj.contains_key("path"));
    assert!(processed_obj.contains_key("islands"));
}
