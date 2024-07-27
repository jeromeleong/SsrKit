# ssrkit

`ssrkit` 是一個強大而靈活的 Rust 庫，專為簡化伺服器端渲染（SSR）的實現過程而設計。它基於 [ssr-rs](https://github.com/Valerioageno/ssr-rs) 專案，進一步擴展了功能和易用性。ssrkit 提供了一套完整的工具，包括參數處理系統、Island 架構支持和模板渲染功能，可以無縫集成到各種 Web 框架中。

## 特性

- **輕量級和高效**: 優化的效能，最小化運行時開銷
- **靈活的參數處理**: 自定義路由參數處理邏輯
- **Island 架構**: 支持部分頁面的客戶端互動，提高應用的互動性
- **強大的模板渲染**: 支持自定義模板和預設模板，滿足各種渲染需求
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
    let result = renderer.render(&path, params, |props| {
        // 這裡可以調用前端的 SSR 函數
        let user = serde_json::from_str::<serde_json::Value>(props).unwrap()["params"]["user"].as_str().unwrap_or("Guest");
        let content = format!("Welcome, {}! Here's a counter for you:", user);
        Ok(serde_json::json!({
            "html": format!("<h1>{}</h1><div data-island=\"Counter\" data-props='{{}}'></div>", content),
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
    Ok(format!(
        r#"<div id="counter" data-island="Counter" data-props='{}'>
            <button>Increment</button>
            <span>{}</span>
        </div>"#,
        serde_json::to_string(props).unwrap(),
        initial_count
    ))
});

island_manager.add_island("Counter", None).unwrap();
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

let result = renderer.render(path, params, |props| {
    // 實現渲染邏輯
});
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
    fn process(&self, island_manager: &Arc<IslandManager>) -> Value {
        // 實現 Island 處理邏輯
    }
}

let island_processor = CombinedIslandProcessor::new()
    .add(IslandDemoProcessor);

let islands_value = island_manager.process_islands(&island_processor);
```

### 與 Web 框架集成

以下是一個使用 Salvo 框架的示例：

```rust
#[handler]
pub async fn handle_ssr(req: &mut Request, res: &mut Response) {
    let path = req.uri().path().to_string();
    let params: std::collections::HashMap<String, String> = req.params().iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    let island_manager = islands::init_island_manager();
    
    let island_processor = CombinedIslandProcessor::new()
        .add(IslandDemoProcessor);

    let islands_value = island_manager.process_islands(&island_processor);

    let renderer: &SsrRenderer = init_renderer(
        || CombinedParamsProcessor::new()
            .add("/user", UserParamsProcessor)
            .add("/blog", BlogParamsProcessor),
        island_manager.clone(),
        get_template(),
    );

    let result = renderer.render(&path, params, |props| {
        // 實現渲染邏輯
    });

    match result {
        Ok(html) => {
            res.render(Text::Html(html));
        },
        Err(e) => {
            res.status_code(StatusCode::INTERNAL_SERVER_ERROR);
            res.render(Text::Plain("Internal Server Error"));
        }
    }
}
```

## 前端集成

ssrkit 需要前端框架的配合才能實現完整的服務器端渲染。以下是一個使用 Svelte 的示例：

### 前端 SSR 入口點 (例如 `ssr.js`)

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
    componentProps.initialCount1 = 10; // 設置第一個計數器的初始值
    componentProps.initialCount2 = 20; // 設置第二個計數器的初始值
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
let result = renderer.render(&path, params, |props| {
    // 呼叫前端的 SSR 函數
    let ssr_result = call_frontend_ssr(props)?;
    
    // 解析前端返回的 JSON
    let rendered: serde_json::Value = serde_json::from_str(&ssr_result)?;
    
    Ok(rendered.to_string())
});
```

這裡的 `call_frontend_ssr` 函數需要根據你的專案設置來實現。它可能涉及調用 Node.js 進程或使用 WebAssembly 來執行 JavaScript 代碼。

### 構建過程

為了使這個流程工作，你需要：

1. 將你的 Svelte 應用編譯成可以在服務器端運行的 JavaScript。
2. 確保 `ssr.js` 文件可以被你的 Rust 後端訪問和執行。
3. 實現一個方法來從 Rust 調用 JavaScript 的 `render` 函數。

這可能需要使用工具如 Webpack 或 Rollup 來打包你的前端代碼，並使用 Node.js 集成或 WebAssembly 來在 Rust 中執行 JavaScript。

## 效能考慮

ssrkit 在設計時考慮了效能：

- **快取**: 考慮在 `ParamsProcessor` 和 `IslandManager` 中實現快取機制。
- **非同步渲染**: 對於大型應用，考慮實現非同步渲染支持。
- **串流渲染**: 考慮實現串流渲染以提高大型頁面的響應速度。

## 最佳實踐

- 盡可能重用 `ParamsProcessor` 和 `IslandManager` 實例。
- 對於靜態內容，考慮實現快取機制。
- 使用 `Island` 架構來最小化客戶端 JavaScript 的大小。
- 根據應用需求，適當分割模板以提高複用性。
- 利用 ssrkit 的模組化設計，根據專案需求自定義和擴展功能。

## 與 ssr-rs 的比較

ssrkit 基於 ssr-rs 專案，但進行了以下改進和擴展：

1. **更完善的參數處理系統**：ssrkit 提供了 `ParamsProcessor` 和 `CombinedParamsProcessor`，支持更靈活的路由特定參數處理。
2. **Island 架構**：ssrkit 引入了 Island 架構，支持部分頁面的客戶端互動，提高了應用的互動性。
3. **模板系統**：ssrkit 提供了更靈活的模板系統，支持自定義模板和預設模板。
4. **Island 管理器**：ssrkit 引入了 `IslandManager`，用於管理和渲染 Island 組件。
5. **前端框架集成**：ssrkit 提供了與前端框架（如 React、Svelte）更好的集成支持。
6. **多執行緒支持**：ssrkit 設計上考慮了多執行緒環境，適用於高併發場景。
7. **擴展性**：ssrkit 的模組化設計使得擴展和自定義功能更加容易。
8. **類型安全**：ssrkit 充分利用 Rust 的類型系統，提供了更好的類型安全保證。

總的來說，ssrkit 在 ssr-rs 的基礎上，提供了更豐富的功能和更好的開發體驗，特別適合構建大型和複雜的 SSR 應用。

## 常見問題解答

1. **Q: ssrkit 可以與哪些 Web 框架一起使用？**
   A: ssrkit 設計為框架無關的，可以與大多數 Rust Web 框架（如 Actix, Rocket, Warp 等）集成。

2. **Q: 如何處理 SEO 問題？**
   A: 使用 ssrkit 的伺服器端渲染可以確保搜尋引擎能夠爬取到完整的頁面內容。確保在模板中包含必要的 meta 標籤。

3. **Q: ssrkit 支持增量靜態再生（ISR）嗎？**
   A: 目前 ssrkit 主要專注於動態 SSR。ISR 可能會在未來版本中考慮支持。

4. **Q: 如何處理大型應用的效能問題？**
   A: 利用 ssrkit 的快取機制、考慮實現非同步渲染，並使用 Island 架構來最小化客戶端 JavaScript。

5. **Q: ssrkit 如何處理前端路由？**
   A: ssrkit 通過與前端框架的集成來處理路由。在伺服器端，你可以根據 URL 選擇適當的組件進行渲染。