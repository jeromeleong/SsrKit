use serde_json::json;
use ssrkit::prelude::*;

#[cfg(not(feature = "island"))]
#[test]
fn test_template_render() {
    // 初始化全局配置
    let config = SsrkitConfig::default();
    ssrkit::config::set_global_config(config);

    // 初始化模板緩存
    ssrkit::template::init_template_cache();

    let template = Template::new();
    let content = json!({
        "html": "<div>test content</div>",
        "css": "body { margin: 0; }",
        "head": "<meta charset=\"UTF-8\">",
        "body": "<script>console.log('loaded');</script>"
    });

    let result = template.render(&content);
    assert!(result.is_ok(), "Template render failed: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<div>test content</div>"));
    assert!(html.contains("body { margin: 0; }"));
    assert!(html.contains("<meta charset=\"UTF-8\">"));
    assert!(html.contains("<script>console.log('loaded');</script>"));
}

#[cfg(feature = "island")]
#[test]
fn test_template_render_with_island() {
    // 初始化全局配置
    let config = SsrkitConfig::default();
    ssrkit::config::set_global_config(config);

    // 初始化模板緩存
    ssrkit::template::init_template_cache();

    // 測試帶有 island 功能的模板渲染
    let template = Template::new();
    let content = json!({
        "html": "<div>test content</div><div data-island=\"island1\" data-name=\"island1\"></div>",
        "css": "body { margin: 0; }",
        "head": "<meta charset=\"UTF-8\">",
        "body": "<script>console.log('loaded');</script>"
    });

    let islands = json!({
        "island1": {
            "id": "island1",
            "html": "<div>island content</div>",
            "instanceId": "instance1"
        }
    });

    let result = template.render(&content, Some(&islands));
    assert!(result.is_ok(), "Template render failed: {:?}", result.err());
    let html = result.unwrap();
    assert!(html.contains("<div>test content</div>"));
    assert!(html.contains("body { margin: 0; }"));
    assert!(html.contains("<meta charset=\"UTF-8\">"));
    assert!(html.contains("<script>console.log('loaded');</script>"));
    assert!(html.contains("<div>island content</div>"));
}
