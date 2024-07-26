# ssrkit

`ssrkit` 是一個強大而靈活的 Rust 庫，專為簡化服務器端渲染（SSR）的實現過程而設計。它提供了一套完整的工具，包括參數處理系統、Island 架構支持和模板渲染功能，可以無縫集成到各種 Web 框架中。

## 特性

- **輕量級和高效**: 優化的性能，最小化運行時開銷
- **靈活的參數處理**: 自定義路由參數處理邏輯
- **Island 架構**: 支持部分頁面的客戶端交互，提高應用的互動性
- **強大的模板渲染**: 支持自定義模板和預設模板，滿足各種渲染需求
- **易於集成**: 設計用於與各種 Rust Web 框架無縫協作
- **可擴展性**: 模塊化設計，易於擴展和自定義
- **線程安全**: 支持多線程環境，適用於高並發場景
- **類型安全**: 利用 Rust 的類型系統確保運行時安全

## 安裝

將以下內容添加到你的 `Cargo.toml` 文件中：

```toml
[dependencies]
ssrkit = { git = "https://github.com/your-username/ssrkit.git" }
```

## 快速開始

以下是一個基本的使用示例，展示了 ssrkit 的核心功能：

```rust
use ssrkit::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

struct BasicParamsProcessor;
impl ParamsProcessor for BasicParamsProcessor {
    fn process(&self, _path: &str, params: &HashMap<String, String>) -> serde_json::Map<String, serde_json::Value> {
        params.iter().map(|(k, v)| (k.clone(), v.clone().into())).collect()
    }
}

fn main() {
    // 初始化模板
    let template = Arc::new(Template::new(r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>SSR Example</title>
            %ssrkit.head%
        </head>
        <body>
            %ssrkit.body%
        </body>
        </html>
    "#));
    
    // 初始化 Island 管理器
    let island_manager = Arc::new(IslandManager::new());
    
    // 註冊一個簡單的 Island 組件
    island_manager.register("Counter", |_, props| {
        let initial_count = props["initialCount"].as_i64().unwrap_or(0);
        Ok(format!(
            r#"<div id="counter" data-island="Counter" data-props='{}'>
                <button>Increment</button>
                <span>{}</span>
            </div>"#,
            serde_json::to_string(props).unwrap(),
            initial_count
        ))
    });
    
    // 創建 SSR 渲染器
    let renderer = create_ssr_renderer(
        || CombinedParamsProcessor::new().add("/", BasicParamsProcessor),
        island_manager,
        template,
    );

    // 模擬請求
    let path = "/example";
    let mut params = HashMap::new();
    params.insert("user".to_string(), "Alice".to_string());
    
    // 執行渲染
    let result = renderer.render(path, params, Some(vec!["Counter"]), |props| {
        let user = props["params"]["user"].as_str().unwrap_or("Guest");
        let content = format!("Welcome, {}! Here's a counter for you:", user);
        Ok(serde_json::json!({
            "html": format!("<h1>{}</h1>{}", content, props["islands"]["Counter"].as_str().unwrap_or("")),
            "css": ".counter { font-weight: bold; }",
            "head": "<meta name='description' content='SSR Example with Counter'>"
        }).to_string())
    });

    println!("Rendered HTML: {}", result.unwrap());
}
```

## 核心概念

### 參數處理 (Params Processing)

參數處理允許你根據路由和請求參數自定義邏輯：

```rust
struct UserParamsProcessor;
impl ParamsProcessor for UserParamsProcessor {
    fn process(&self, path: &str, params: &HashMap<String, String>) -> serde_json::Map<String, serde_json::Value> {
        let mut processed = serde_json::Map::new();
        if let Some(user_id) = params.get("id") {
            processed.insert("user_id".to_string(), user_id.clone().into());
            processed.insert("is_admin".to_string(), (user_id == "admin").into());
        }
        processed
    }
}
```

### Island 架構

Island 架構允許你在服務器端渲染的頁面中嵌入可交互的客戶端組件：

```rust
island_manager.register("Counter", |_, props| {
    let initial_count = props["initialCount"].as_i64().unwrap_or(0);
    Ok(format!(
        r#"<div id="counter" data-island="Counter" data-props='{}'>
            <button>Increment</button>
            <span>{}</span>
        </div>"#,
        serde_json::to_string(props).unwrap(),
        initial_count
    ))
});

island_manager.add_island("Counter", "<div>Counter placeholder</div>").unwrap();
```

### 模板渲染

ssrkit 支持靈活的模板渲染，你可以使用預設模板或創建自定義模板：

```rust
let template = Arc::new(Template::new(r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>My SSR App</title>
        %ssrkit.head%
    </head>
    <body>
        %ssrkit.body%
    </body>
    </html>
"#));
```

### SSR 渲染器

SSR 渲染器是 ssrkit 的核心，它協調參數處理、Island 渲染和模板填充：

```rust
let renderer = create_ssr_renderer(
    params_processor_init,
    island_manager,
    template,
);

let result = renderer.render(path, params, island_ids, |props| {
    // 實現渲染邏輯
});
```

## 高級使用

### 自定義模板

你可以創建自己的模板結構體來實現更複雜的渲染邏輯：

```rust
struct MyTemplate {
    base_html: String,
}

impl MyTemplate {
    fn new(base_html: &str) -> Self {
        Self { base_html: base_html.to_owned() }
    }

    fn render(&self, content: &Value, islands: &Value) -> Result<String, String> {
        // 實現自定義渲染邏輯
    }
}
```

### 複雜的參數處理

對於更複雜的應用，你可以組合多個參數處理器：

```rust
let params_processor = CombinedParamsProcessor::new()
    .add("/user/", UserParamsProcessor)
    .add("/product/", ProductParamsProcessor)
    .add("/", DefaultParamsProcessor);
```

### 集成測試

ssrkit 設計時考慮了可測試性。以下是一個簡單的集成測試示例：

```rust
#[test]
fn test_ssr_rendering() {
    // 初始化測試環境
    let template = Arc::new(Template::new("..."));
    let island_manager = Arc::new(IslandManager::new());
    let renderer = create_ssr_renderer(
        || TestParamsProcessor,
        island_manager,
        template,
    );

    // 執行測試
    let result = renderer.render("/test", HashMap::new(), None, |props| {
        Ok(serde_json::json!({"html": "<div>Test Content</div>"}).to_string())
    });

    assert!(result.is_ok());
    assert!(result.unwrap().contains("Test Content"));
}
```

## 性能考慮

ssrkit 在設計時考慮了性能：

- **緩存**: 考慮在 `ParamsProcessor` 和 `IslandManager` 中實現緩存機制。
- **異步渲染**: 對於大型應用，考慮實現異步渲染支持。
- **流式渲染**: 考慮實現流式渲染以提高大型頁面的響應速度。

## 最佳實踐

- 盡可能重用 `ParamsProcessor` 和 `IslandManager` 實例。
- 對於靜態內容，考慮實現緩存機制。
- 使用 `Island` 架構來最小化客戶端 JavaScript 的大小。
- 根據應用需求，適當分割模板以提高複用性。

## 常見問題解答

1. **Q: ssrkit 可以與哪些 Web 框架一起使用？**
   A: ssrkit 設計為框架無關的，可以與大多數 Rust Web 框架（如 Actix, Rocket, Warp 等）集成。

2. **Q: 如何處理 SEO 問題？**
   A: 使用 ssrkit 的服務器端渲染可以確保搜索引擎能夠爬取到完整的頁面內容。確保在模板中包含必要的 meta 標籤。

3. **Q: ssrkit 支持增量靜態再生（ISR）嗎？**
   A: 目前 ssrkit 主要專注於動態 SSR。ISR 可能會在未來版本中考慮支持。

## 貢獻

我們歡迎社區貢獻！如果你發現了 bug 或有新的功能建議，請開啟一個 issue 或提交一個 pull request。

## 路線圖

- [ ] 實現異步渲染支持
- [ ] 添加流式渲染功能
- [ ] 改進文檔和示例
- [ ] 添加更多集成測試
- [ ] 性能優化和基準測試

## 許可證

本項目暫時未有許可證