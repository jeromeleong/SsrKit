# ssrkit

`ssrkit` 是一個強大而靈活的 Rust 庫，專為簡化伺服器端渲染（SSR）的實現過程而設計。它基於 [ssr-rs](https://github.com/Valerioageno/ssr-rs) 專案，進一步擴展了功能和易用性。ssrkit 提供了一套完整的工具，包括參數處理系統、Island 架構支持和模板渲染功能，可以無縫集成到各種 Web 框架中。

## 特性

- **輕量級和高效**: 優化的效能，最小化運行時開銷
- **靈活的參數處理**: 自定義路由參數處理邏輯
- **Island 架構**: 支持部分頁面的客戶端互動，提高應用的互動性
- **模板渲染**: 內建模板系統，支持自定義內容插入
- **易於集成**: 設計用於與各種 Rust Web 框架和前端框架無縫協作
- **可擴展性**: 模組化設計，易於擴展和自定義
- **執行緒安全**: 支持多執行緒環境，適用於高併發場景
- **類型安全**: 利用 Rust 的類型系統確保運行時安全

## 安裝

將以下內容添加到你的 `Cargo.toml` 檔案中：

```toml
[dependencies]
ssrkit = { git = "https://git.leongfamily.net/jerome/ssrkit.git" }
```

## 快速開始

以下是一個基本的使用示例，展示了 ssrkit 的核心功能：

```rust
use ssrkit::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;

struct BasicParamsProcessor;
impl ParamsProcessor for BasicParamsProcessor {
    fn process(&self, _path: &str, params: &HashMap<String, String>) -> serde_json::Map<String, Value> {
        params.iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect()
    }
}

struct ExampleIslandProcessor;
impl IslandProcessor for ExampleIslandProcessor {
    fn process(&self, island_manager: &Arc<IslandManager>, context: &ProcessContext) -> Value {
        let mut islands = serde_json::Map::new();

        if context.path == "/example" {
            // Counter Island
            let counter = get_or_render_island("counter", || {
                island_manager.render_island("Counter", &json!({
                    "initialCount": 0,
                    "client": "load"
                })).unwrap_or_default()
            });
            
            islands.insert("counter".to_string(), json!({
                "id": "Counter",
                "html": counter
            }));
        }

        Value::Object(islands)
    }
}

fn main() {
    // 初始化 SSR 組件
    init_ssr(
        || Box::new(CombinedParamsProcessor::new().add("/", BasicParamsProcessor)),
        || {
            let manager = IslandManager::new();
            
            manager.register("Counter", |_, props| {
                let initial_count = props["initialCount"].as_i64().unwrap_or(0);
                let instance_id = props["instanceId"].as_str().unwrap_or("");
                Ok(format!(
                    r#"<div id="{}" data-island="Counter" data-props='{}'>
                        <button>遞增</button>
                        <span>{}</span>
                    </div>"#,
                    instance_id,
                    serde_json::to_string(props).unwrap(),
                    initial_count
                ))
            });
            
            manager.add_island("Counter", Some(json!({"initialCount": 0}))).unwrap();
            
            manager
        },
        Template::new,
        None, // 可選的 SsrkitConfig
    );

    let renderer = get_renderer();

    // 模擬請求
    let path = "/example";
    let mut params = HashMap::new();
    params.insert("user".to_string(), "Alice".to_string());

    // 執行渲染
    let result = renderer.process_and_render(
        &ExampleIslandProcessor,
        path, 
        params,
        |props| {
            let parsed_props: Value = serde_json::from_str(props).unwrap();
            let user = parsed_props["params"]["user"].as_str().unwrap_or("訪客");
            let content = format!("歡迎，{}！這是一個互動計數器：", user);
            
            Ok(json!({
                "html": format!(
                    r#"<h1>{}</h1>
                    <div data-island="Counter" data-name="counter" data-props='{{"initialCount": 0}}'></div>"#,
                    content
                ),
                "css": ".counter { font-weight: bold; }",
                "head": "<meta name='description' content='SSRKit 示例與計數器'>"
            }).to_string())
        }
    );

    match result {
        Ok(html) => println!("渲染的 HTML：\n{}", html),
        Err(e) => println!("渲染錯誤：{}", e),
    }
}
```

## 核心概念

### 初始化 SSR

初始化 SSR 是使用 ssrkit 的第一步，它設置了整個 SSR 系統的核心組件：

```rust
init_ssr(
    params_processor_init: impl FnOnce() -> Box<dyn ParamsProcessor>,
    island_manager_init: impl FnOnce() -> IslandManager,
    template_init: impl FnOnce() -> Template,
    config: Option<&SsrkitConfig>,
)
```

- `params_processor_init`: 初始化參數處理器
- `island_manager_init`: 初始化 Island 管理器
- `template_init`: 初始化模板
- `config`: 可選的 SSRKit 配置

### 參數處理 (Params Processing)

參數處理允許你根據路由和請求參數自定義邏輯：

```rust
struct UserParamsProcessor;
impl ParamsProcessor for UserParamsProcessor {
    fn process(&self, _path: &str, params: &HashMap<String, String>) -> serde_json::Map<String, serde_json::Value> {
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

Island 架構允許你在伺服器端渲染的頁面中嵌入可互動的客戶端組件：

```rust
island_manager.register("Counter", |_, props| {
    let initial_count = props["initialCount"].as_i64().unwrap_or(0);
    let instance_id = props["instanceId"].as_str().unwrap_or("");
    Ok(format!(
        r#"<div id="{}" data-island="Counter" data-props='{}'>
            <button>遞增</button>
            <span>{}</span>
        </div>"#,
        instance_id,
        serde_json::to_string(props).unwrap(),
        initial_count
    ))
});

island_manager.add_island("Counter", Some(json!({"initialCount": 0}))).unwrap();
```

### 模板渲染

ssrkit 使用內建的模板系統來組合最終的 HTML 輸出：

```rust
let template = Template::new();
```

模板系統會自動處理 HTML、CSS 和額外的頭部內容。

### SSR 渲染器

SSR 渲染器是 ssrkit 的核心，它協調參數處理、Island 渲染和模板填充：

```rust
let renderer = get_renderer();

let result = renderer.process_and_render(
    &island_processor,
    path,
    params,
    |props| {
        // 實現渲染邏輯
        Ok(json!({
            "html": "<h1>你好，世界！</h1>",
            "css": ".greeting { color: blue; }",
            "head": "<meta name='description' content='我的 SSR 頁面'>"
        }).to_string())
    }
);
```

## 進階使用

### 自定義參數處理器

對於更複雜的應用，你可以組合多個參數處理器：

```rust
let params_processor = CombinedParamsProcessor::new()
    .add("/user", UserParamsProcessor)
    .add("/blog", BlogParamsProcessor);
```

### Island 處理器

Island 處理器允許你在渲染過程中動態處理 Island 組件：

```rust
struct IslandDemoProcessor;

impl IslandProcessor for IslandDemoProcessor {
    fn process(&self, island_manager: &Arc<IslandManager>, context: &ProcessContext) -> Value {
        let mut islands = serde_json::Map::new();
        if context.path == "/demo" {
            let counter = get_or_render_island("counter", || {
                island_manager.render_island("Counter", &json!({"initialCount": 5})).unwrap_or_default()
            });
            islands.insert("counter".to_string(), json!({"id": "Counter", "html": counter}));
        }
        Value::Object(islands)
    }
}
```

### 與 Web 框架集成

以下是一個使用 Salvo 框架的示例：

```rust
use salvo::prelude::*;
use ssrkit::prelude::*;

#[handler]
pub async fn handle_ssr(req: &mut Request, res: &mut Response) {
    let path = req.uri().path().to_string();
    let params: std::collections::HashMap<String, String> = req.params().iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let renderer = get_renderer();
    let island_processor = IslandDemoProcessor;

    let result = renderer.process_and_render(
        &island_processor,
        &path,
        params,
        |props| {
            // 實現渲染邏輯
            Ok(json!({
                "html": "<h1>你好，Salvo！</h1>",
                "css": ".greeting { color: green; }",
                "head": "<meta name='description' content='Salvo SSR 示例'>"
            }).to_string())
        }
    );

    match result {
        Ok(html) => {
            res.render(Text::Html(html));
        },
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Text::Plain(format!("內部伺服器錯誤：{}", e)));
        }
    }
}
```

## 前端集成

ssrkit 需要前端框架的配合才能實現完整的伺服器端渲染。以下是一個使用 Svelte 的示例：

### 前端 SSR 入口點（例如 `ssr.js`）

```javascript
import App from './App.svelte';
import * as routes from './routes';

export function render(props) {
  const { url, params, islands } = JSON.parse(props);
  
  let component = routes.NotFound;
  let componentProps = { ...params };

  // 根據 URL 選擇適當的組件
  if (url === '/') {
    component = routes.Index;
  } else if (url === '/about') {
    component = routes.About;
  } else if (url.startsWith('/blog')) {
    component = routes.BlogPost;
  } else if (url.startsWith('/user')) {
    component = routes.User;
  } else if (url === '/island-demo') {
    component = routes.IslandDemo;
    componentProps.initialCount = 10; // 設置計數器的初始值
  }

  const rendered = App.render({
    url,
    component,
    props: componentProps,
    islands
  });

  return JSON.stringify({
    html: rendered.html,
    css: rendered.css.code,
    head: rendered.head
  });
}
```

### 在 ssrkit 中使用前端渲染

在 Rust 後端中，你可以這樣使用前端的 SSR 渲染：

```rust
let result = renderer.process_and_render(
    &island_processor,
    &path,
    params,
    |props| {
        // 調用前端的 SSR 函數
        let ssr_result = call_frontend_ssr(props)?;
        
        // 解析前端返回的 JSON
        let rendered: serde_json::Value = serde_json::from_str(&ssr_result)?;
        
        Ok(rendered.to_string())
    }
);
```

這裡的 `call_frontend_ssr` 函數需要根據你的專案設置來實現。它可能涉及調用 Node.js 進程或使用 WebAssembly 來執行 JavaScript 程式碼。

## 效能考慮

ssrkit 在設計時考慮了效能：

- **快取**: `get_or_render_island` 函數提供了內建的快取機制，避免重複渲染相同的 Island。
- **並行處理**: 雖然目前 ssrkit 不直接支持非同步渲染，但你可以在 `IslandProcessor` 的實現中使用並行處理技術來提高效能。
- **選擇性水合**: Island 架構允許選擇性地水合頁面的特定部分，減少客戶端 JavaScript 的大小和執行時間。

### 效能優化技巧

1. **快取策略優化**:
   - 使用粗粒度的時間戳來增加快取命中率。
   - 實現自定義快取鍵生成策略，排除頻繁變化的數據。

2. **減少動態內容**:
   - 將頁面分為靜態部分和動態部分，靜態部分可以長期快取。
   - 使用客戶端 JavaScript 更新動態內容，如時間戳。

3. **使用 ETags**:
   - 實現 ETags 來允許客戶端快取頁面，只有當內容真正變化時才發送新的回應。

4. **增加快取時間**:
   - 如果內容不需要即時更新，可以增加快取的有效期。

5. **批量處理**:
   - 在 `IslandProcessor` 中實現批量處理邏

## 與 ssr-rs 的比較

ssrkit 基於 ssr-rs 專案，但進行了以下改進和擴展：

1. **更完善的參數處理系統**:
   - ssrkit 提供了 `ParamsProcessor` 和 `CombinedParamsProcessor`，支持更靈活的路由特定參數處理。
   - 允許開發者根據不同路由實現自定義的參數處理邏輯。

2. **Island 架構**:
   - ssrkit 引入了 Island 架構，支持部分頁面的客戶端互動，提高了應用的互動性。
   - 允許在伺服器端渲染的頁面中嵌入可互動的客戶端組件。

3. **模板系統**:
   - ssrkit 提供了更靈活的模板系統，支持自定義模板和預設模板。
   - 允許更細緻的控制over HTML結構、CSS和JavaScript注入。

4. **Island 管理器**:
   - ssrkit 引入了 `IslandManager`，用於管理和渲染 Island 組件。
   - 提供了一個集中的方式來註冊、管理和渲染 Island 組件。

5. **前端框架集成**:
   - ssrkit 提供了與前端框架（如 React、Svelte）更好的集成支持。
   - 簡化了前端和後端之間的數據傳遞和狀態管理。

6. **多執行緒支持**:
   - ssrkit 設計上考慮了多執行緒環境，適用於高併發場景。
   - 提供了執行緒安全的數據結構和操作。

7. **擴展性**:
   - ssrkit 的模組化設計使得擴展和自定義功能更加容易。
   - 提供了更多的鉤子（hooks）和擴展點。

8. **類型安全**:
   - ssrkit 充分利用 Rust 的類型系統，提供了更好的類型安全保證。
   - 減少了運行時錯誤，提高了代碼的可靠性。

9. **效能優化**:
   - ssrkit 提供了更多的內建效能優化選項。
   - 包括更細緻的快取控制和選擇性水合等特性。

10. **開發者體驗**:
    - ssrkit 提供了更豐富的文檔和示例。
    - 設計了更直觀和易用的 API，減少了學習曲線。

## 常見問題解答

1. **Q: ssrkit 可以與哪些 Web 框架一起使用？**
   A: ssrkit 設計為框架無關的，可以與大多數 Rust Web 框架（如 Actix, Rocket, Warp 等）集成。

2. **Q: 如何處理 SEO 問題？**
   A: 使用 ssrkit 的伺服器端渲染可以確保搜尋引擎能夠爬取到完整的頁面內容。確保在模板中包含必要的 meta 標籤。

3. **Q: ssrkit 支持增量靜態再生（ISR）嗎？**
   A: 目前 ssrkit 主要專注於動態 SSR。ISR 可能會在未來版本中考慮支持。

4. **Q: 如何處理大型應用的效能問題？**
   A: 利用 ssrkit 的快取機制、考慮在 `IslandProcessor` 中實現並行處理，並使用 Island 架構來最小化客戶端 JavaScript。

5. **Q: ssrkit 如何處理前端路由？**
   A: ssrkit 通過與前端框架的集成來處理路由。在伺服器端，你可以根據 URL 選擇適當的組件進行渲染。

6. **Q: 如何自定義 Island 的客戶端行為？**
   A: Island 的客戶端行為應在前端框架中實現。ssrkit 負責伺服器端渲染和初始狀態的傳遞。

7. **Q: ssrkit 是否支持部分頁面更新？**
   A: ssrkit 主要關注完整頁面的 SSR。部分頁面更新通常應由客戶端 JavaScript 處理。

8. **Q: 如何處理認證和授權？**
   A: 認證和授權邏輯應在 `ParamsProcessor` 或你的 Web 框架中實現。ssrkit 可以根據這些邏輯的結果來渲染相應的內容。

## 貢獻

我們歡迎社區貢獻！如果你有任何改進建議或發現了 bug，請開啟一個 issue 或提交一個 pull request。

## 授權

ssrkit 暫未選擇授權方式

## 致謝

特別感謝 ssr-rs 專案的開發者，他的工作為 ssrkit 奠定了基礎。同時也感謝所有為 ssrkit 做出貢獻的開發者。