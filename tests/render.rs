use ssrkit::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "island")]
use serde_json::json;

#[cfg(not(feature = "island"))]
#[test]
fn test_render() {
    // 初始化全局配置
    let config = SsrkitConfig::default();
    ssrkit::config::set_global_config(config.clone());

    // 初始化全局狀態
    let cache = Cache::new(|config| config.get_global_state_cache_size());
    let session_duration = std::time::Duration::from_secs(3600);
    init_global_state(cache, config, session_duration);

    // 初始化模板緩存
    ssrkit::template::init_template_cache();

    let params_processor = Box::new(CombinedParamsProcessor::new());
    let template = Arc::new(Template::new());
    let renderer = SsrRenderer::new(params_processor, template);

    let path = "/test";
    let params = HashMap::new();
    let render_fn: Box<dyn FnOnce(&str) -> Result<String, String>> = Box::new(|props| {
        let json_props =
            serde_json::from_str::<serde_json::Value>(props).map_err(|e| e.to_string())?;
        let content = format!("test content with props: {}", json_props);
        let result = serde_json::json!({
            "html": content,
            "css": "",
            "head": "",
            "body": ""
        });
        Ok(result.to_string())
    });

    let result = renderer.render(path, params, render_fn);
    if let Err(ref e) = result {
        println!("Render error: {}", e);
        assert!(false);
    }
    let (html, _) = result.unwrap();
    assert!(html.contains("test content with props:"));
}

#[cfg(feature = "island")]
#[test]
fn test_render_with_island() {
    // 測試帶有 island 功能的渲染
    // 初始化全局配置
    let config = SsrkitConfig::default();
    ssrkit::config::set_global_config(config.clone());

    // 初始化全局狀態
    let cache = Cache::new(|config| config.get_global_state_cache_size());
    let session_duration = std::time::Duration::from_secs(3600);
    init_global_state(cache, config, session_duration);

    // 初始化模板緩存
    ssrkit::template::init_template_cache();

    let params_processor = Box::new(CombinedParamsProcessor::new());
    let template = Arc::new(Template::new());
    let island_manager = Arc::new(IslandManager::new());
    let renderer = SsrRenderer::new(params_processor, island_manager, template);

    let path = "/test";
    let params = HashMap::new();
    let render_fn: Box<dyn FnOnce(&str) -> Result<String, String>> = Box::new(|props| {
        let json_props =
            serde_json::from_str::<serde_json::Value>(props).map_err(|e| e.to_string())?;
        let content = format!("test content with props: {}", json_props);
        let result = json!({
            "html": content,
            "css": "",
            "head": "",
            "body": ""
        });
        Ok(result.to_string())
    });

    let processor = CombinedIslandProcessor::new();
    let result = renderer.render(path, params, render_fn, &processor);
    if let Err(ref e) = result {
        println!("Render error: {}", e);
        assert!(false);
    }
    let (html, _) = result.unwrap();
    assert!(html.contains("test content with props:"));
}
