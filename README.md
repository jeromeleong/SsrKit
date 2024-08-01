# ssrkit

![Crates.io](https://img.shields.io/crates/v/ssrkit)
[![rust-clippy analyze](https://github.com/jeromeleong/SsrKit/actions/workflows/rust-clippy.yml/badge.svg)](https://github.com/jeromeleong/SsrKit/actions/workflows/rust-clippy.yml)
![License](https://img.shields.io/crates/l/ssrkit)

`ssrkit` 是一個強大且靈活的 Rust 函式庫，專為簡化伺服器端渲染（SSR）的實作流程而設計。它基於 [ssr-rs](https://github.com/Valerioageno/ssr-rs) 項目，進一步擴展了功能和易用性。 ssrkit 提供了一套完整的工具，包括參數處理系統、Island 架構支援和模板渲染功能，可無縫整合到各種 Web 框架中。

## 特性

- **輕量級和高效**: 最佳化的效能，最小化運行時開銷
- **靈活的參數處理**: 自訂路由參數處理邏輯
- **Island 架構**: 支援部分頁面的客戶端交互，提高應用程式的交互性
- **範本渲染**: 內建模板系統，支援自訂內容插入
- **易於整合**: 設計用於與各種 Rust Web 框架和前端框架無縫協作
- **可擴展性**: 模組化設計，易於擴展和自訂
- **執行緒安全性**: 支援多執行緒環境，適用於高並發場景
- **型別安全**: 利用 Rust 的型別系統確保執行時間安全

## 安裝

將以下內容加入你的 `Cargo.toml` 檔案中：

```toml
[dependencies]
ssrkit = "0.1.0"
ssr = "0.5.7" # 確保使用與 ssrkit 相容的 ssr-rs 或其他 ssr 庫
```

注意：ssrkit 依賴 ssr-rs 和其他 SSR 相關函式庫。請確保你的專案中包含了所有必要的依賴。

## 快速開始

以下是一個基本的使用範例，展示了 ssrkit 的核心功能：

```rust
use serde_json::json;
use ssr::Ssr;
use ssrkit::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

struct BasicParamsProcessor;
impl ParamsProcessor for BasicParamsProcessor {
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

struct ExampleIslandProcessor;
impl IslandProcessor for ExampleIslandProcessor {
    fn process(&self, island_manager: &Arc<IslandManager>, context: &ProcessContext) -> Value {
        let mut islands = serde_json::Map::new();

        if context.path == "/example" {
            // Counter Island
            let counter = get_or_render_island("counter", || {
                island_manager
                    .render_island(
                        "Counter",
                        &json!({
                            "initialCount": 0,
                            "client": "load"
                        }),
                    )
                    .unwrap_or_default()
            });

            islands.insert(
                "counter".to_string(),
                json!({
                "id": "Counter",
                "html": counter
                }),
            );
        }

        Value::Object(islands)
    }
}

fn main() {
    // 初始化 SSR 元件
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

            manager
                .add_island("Counter", Some(json!({"initialCount": 0})))
                .unwrap();

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
    let result = renderer.process_and_render(&ExampleIslandProcessor, path, params, |props| {
        let parsed_props: Value = serde_json::from_str(props).unwrap();
        let user = parsed_props["params"]["user"].as_str().unwrap_or("訪客");
        let content = format!("歡迎，{}！這是一個互動計數器：", user);

        // 使用 ssr-rs 進行實際的 SSR 渲染
        let ssr = Ssr::new("path/to/your/frontend/bundle.js", "render").unwrap();
        let rendered = ssr.render(props).unwrap();

        Ok(json!({
        "html": format!(
        r#"<h1>{}</h1>
            <div data-island="Counter" data-name="counter" data-props='{{"initialCount": 0}}'></div>
        {}"#,
        content,
        rendered
        ),
        "css": ".counter { font-weight: bold; }",
        "head": "<meta name='description' content='SSRKit 範例與計數器'>"
        })
        .to_string())
    });

    match result {
        Ok(html) => println!("渲染的 HTML：\n{}", html),
        Err(e) => println!("渲染錯誤：{}", e),
    }
}
```

## 核心概念

### 初始化 SSR

初始化 SSR 是使用 ssrkit 的第一步，它設定了整個 SSR 系統的核心元件：

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

參數處理允許你根據路由和請求參數自訂邏輯：

```rust
struct UserParamsProcessor;
impl ParamsProcessor for UserParamsProcessor {
    fn process(
        &self,
        _path: &str,
        params: &HashMap<String, String>,
    ) -> serde_json::Map<String, serde_json::Value> {
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
    // 實作渲染邏輯，這裡通常會呼叫 ssr-rs 的 SSR 功能
    // let ssr = Ssr::new("path/to/your/frontend/bundle.js", "render").unwrap();
    // let rendered = ssr.render(props).unwrap();  
    //
    Ok(json!({
            "html": "<h1>你好，世界！</h1>",
            "css": ".greeting { color: blue; }",
            "head": "<meta name='description' content='我的 SSR 頁面'>"
        }).to_string())
    }
);
```

## 進階使用

### 自訂參數處理器

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

以下是一個使用 Salvo 框架的範例：

```rust
use salvo::prelude::*;
use ssrkit::prelude::*;
use ssr::Ssr;

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
            // 使用 ssr-rs 進行實際的 SSR 渲染
            let ssr = Ssr::new("path/to/your/frontend/bundle.js", "render").unwrap();
            let rendered = ssr.render(props).unwrap();

            Ok(json!({
                "html": format!("<h1>你好，Salvo！</h1>{}", rendered),
                "css": ".greeting { color: green; }",
                "head": "<meta name='description' content='Salvo SSR 範例'>"
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

ssrkit 需要前端框架的配合才能達到完整的伺服器端渲染。以下是一個使用 Svelte 的範例：

### 前端 SSR 入口點（例如 `ssr.js`）

```javascript
import App from './App.svelte';
import * as routes from './routes';

export function render(props) {
    const { url, params, islands } = JSON.parse(props);

    let component = routes.NotFound;
    let componentProps = { ...params };

    // 根據 URL 選擇適當的元件
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
        componentProps.initialCount = 10; // 設定計數器的初始值
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
        // 使用 ssr-rs 呼叫前端的 SSR 函數
        let ssr = Ssr::new("path/to/your/frontend/bundle.js", "render").unwrap();
        let ssr_result = ssr.render(props)?;

        // 解析前端傳回的 JSON
        let rendered: serde_json::Value = serde_json::from_str(&ssr_result)?;

        Ok(rendered.to_string())
    }
);
```

## 效能考慮

- **選擇性水合**: Island 架構允許選擇性地水合頁面的特定部分，減少客戶端 JavaScript 的大小和執行時間。
- **串流 SSR**: 雖然 ssrkit 本身不直接提供串流 SSR，但可以與支援串流輸出的 Web 框架結合使用，提高首次內容繪製（FCP）時間。

### 效能最佳化技巧

1. **快取策略優化**:
 - 使用粗粒度的時間戳記來增加快取命中率。
 - 實作自訂快取鍵產生策略，排除頻繁變化的資料。

2. **減少動態內容**:
 - 將頁面分為靜態部分和動態部分，靜態部分可以長期快取。
 - 使用客戶端 JavaScript 更新動態內容，例如時間戳記。

3. **使用 ETags**:
 - 實作 ETags 來允許客戶端快取頁面，只有當內容真正變化時才發送新的回應。

4. **增加快取時間**:
 - 如果內容不需要即時更新，可以增加快取的有效期限。

5. **批量處理**:
 - 在 `IslandProcessor` 中實作批次處理邏輯，減少資料庫查詢次數。

6. **優化前端程式碼**:
 - 確保前端 SSR 程式碼高效，避免不必要的計算和渲染。

## 與 ssr-rs 的關係

ssrkit 基於 ssr-rs 項目，並對其進行了擴展。以下是 ssrkit 與 ssr-rs 的主要區別和改進：

1. **參數處理系統**:
 - ssrkit 引入了更靈活的參數處理系統，允許根據路由自訂參數處理邏輯。

2. **Island 架構**:
 - ssrkit 新增了 Isl​​and 架構支持，實現了更細粒度的客戶端互動。

3. **模板系統**:
 - ssrkit 提供了內建的模板系統，簡化了 HTML、CSS 和頭部內容的組合。

4. **狀態管理**:
 - ssrkit 引入了全域狀態管理，便於在不同元件間共享資料。

5. **擴充性**:
 - ssrkit 設計了更多的擴充點，便於與其他函式庫和框架整合。

6. **型別安全**:
 - ssrkit 更多地利用了 Rust 的類型系統，提供了更強的類型安全保證。

雖然 ssrkit 增加了這些特性，但它仍然依賴 ssr-rs 作為底層 SSR 引擎。在使用 ssrkit 時，你需要同時引入 ssr-rs 作為依賴。

## 最佳實踐

1. **元件化設計**: 將應用程式分解為小型、可重複使用的元件，以便於維護和測試。

2. **提前準備資料**: 在呼叫 SSR 渲染之前，盡可能準備好所需的所有資料。

3. **錯誤處理**: 實現全面的錯誤處理策略，確保在 SSR 失敗時有適當的回退機制。

4. **效能監控**: 使用效能監控工具追蹤 SSR 的執行時間和資源使用情況。

5. **程式碼分割**: 利用動態導入和懶載入技術減少初始載入時間。

6. **SSR 與 CSR 結合**: 對於不需要 SEO 的頁面部分，請考慮使用客戶端渲染。

7. **合理使用 Islands**: 只對真正需要互動的元件使用 Island 架構。

## 常見問題解答

1. **Q: ssrkit 可以與哪些 Web 框架一起使用？ **
 A: ssrkit 設計為框架無關的，可以與大多數 Rust Web 框架（如 Actix, Rocket, Warp 等）整合。

2. **Q: 如何處理 SEO 問題？ **
 A: 使用 ssrkit 的伺服器端渲染可以確保搜尋引擎能夠爬取到完整的頁面內容。確保在範本中包含必要的 meta 標籤。

3. **Q: ssrkit 支援增量靜態再生（ISR）嗎？ **
 A: 目前 ssrkit 主要專注於動態 SSR。 ISR 可能會在未來版本中考慮支援。

4. **Q: 如何處理大型應用的效能問題？ **
 A: 利用 ssrkit 的快取機制、考慮在 `IslandProcessor` 中實作並行處理，並使用 Island 架構來最小化客戶端 JavaScript。

5. **Q: ssrkit 如何處理前端路由？ **
 A: ssrkit 透過與前端框架的整合來處理路由。在伺服器端，你可以根據 URL 選擇適當的元件進行渲染。

6. **Q: 如何自訂 Island 的客戶行為？ **
 A: Island 的客戶行為應在前端框架中實現。 ssrkit 負責伺服器端渲染和初始狀態的傳遞。

7. **Q: ssrkit 是否支援部分頁面更新？ **
 A: ssrkit 主要關注完整頁面的 SSR。部分頁面更新通常應由客戶端 JavaScript 處理。

8. **Q: 如何處理認證和授權？ **
 A: 認證和授權邏輯應在 `ParamsProcessor` 或你的 Web 框架中實作。 ssrkit 可以根據這些邏輯的結果來渲染對應的內容。

## 貢獻

我們歡迎社區貢獻！如果你有任何改進建議或發現了 bug，請開啟一個 issue 或提交一個 pull request。

## 許可

ssrkit 使用 MIT 許可證。

## 致謝

特別感謝 ssr-rs 專案的開發者，他的工作為 ssrkit 奠定了基礎。

## 相關項目

- [ssr-rs](https://github.com/Valerioageno/ssr-rs): ssrkit 的基礎 SSR 引擎
- [salvo](https://github.com/salvo-rs/salvo): 一個相容於 ssrkit 的 Rust Web 框架
- [Svelte](https://svelte.dev/): 一個受歡迎的前端框架，可以與 ssrkit 搭配使用

## 更新日誌

請查看 [CHANGELOG.md](CHANGELOG.md) 檔案以了解最新的變更和版本資訊。

## 聯絡我們

如果你有任何問題或建議，可以透過以下方式聯絡我：

- GitHub Issues: [ssrkit issues](https://github.com/jeromeleong/ssrkit/issues)
- Email: jeromeleong1998@gmail.com

感謝你使用 ssrkit！我們期待看到你用它構建的出色應用。