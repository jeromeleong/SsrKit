# SsrKit

![Crates.io](https://img.shields.io/crates/v/ssrkit)
![License](https://img.shields.io/crates/l/ssrkit)

`SsrKit` 是一個用於簡化伺服器端渲染（SSR）使用的庫。本項目以 [ssr-rs](https://github.com/Valerioageno/ssr-rs)為核心，為其進一步擴展了功能和易用性。

## 什麼是 `ssr-rs`？
`ssr-rs` 用嵌入版本的v8引擎為JS 前端和 Rust 後端提供服務端渲染的橋樑。
你可以使用 Vue、React等JavaScript 框架完成前端，然後通過`ssr-rs`的v8 將內容轉譯給Rust 進行後續處理。
`ssr-rs`只提供最基本的功能，為了獲得更好的SSR 開發體驗，`SsrKit`就因此出現

## SsrKit 的特性
- 全局狀態管理：提供了基本的全局狀態管理功能，包括Session 管理和Cookie 管理
- 基於路由的參數處理： 提供了基本的路由參數處理機制，如`/blog/[id]`中`[id]`參數的處理，並根據參數可以輸出不同的內容。
- LRU的緩存機制：通過「最近最少使用」的緩存策略緩存常用的渲染結果，可以減少重複計算，從而加快響應速度。
- Island 架構支持：(需要`island`feature)支持 Island 架構，這是一種用於實現局部更新和增量渲染的技術。通過 Island 架構，你可以將頁面拆分成多個獨立的 Island，每個 Island 可以獨立渲染和更新，從而提升用戶體驗。

## 安裝

將以下內容加入你的 `Cargo.toml` 檔案中：

```toml
[dependencies]
ssrkit = { version = "0.1.2" , features = ["island"] }
ssr = "0.5.8" # 確保使用與 ssrkit 相容的 ssr-rs 或其他 ssr 庫
```

注意：ssrkit 依賴 ssr-rs 和其他 SSR 相關函式庫。請確保你的專案中包含了所有必要的依賴。

## 快速開始

以下是一個快速開始的使用範例，簡單展示了 SsrKit 如何助力伺服器端渲染（SSR）。

```rust
use ssrkit::prelude::*;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use serde_json::{Map, Value};

// 通過使用 #[params_handle] 宏來簡化 ParamsProcessor 的實現
#[params_handle]
fn BlogParamsProcessor(path: &str, params: &HashMap<String, String>) -> Map<String, Value> {
    // 將請求參數轉換為 serde_json::Map
    params
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect()
}

// (需要`island`feature) 使用 #[island_handle] 宏來簡化 IslandProcessor 的實現
#[island_handle]
fn IslandDemoProcessor(&self, island_manager: &Arc<IslandManager>, context: &ProcessContext) -> Value {
    // 返回一個空的 JSON 對象
    serde_json::json!({})
}

// 定義一個簡單的 Island 渲染函數
fn render_counter(_id: &str, _props: &Value) -> Result<String, String> {
    Ok("<div>Counter Island</div>".to_string())
}

// 初始化 SSR 組件
pub fn init_ssr_components() {
    // 初始化 SSR 渲染器
    SsrInitializer::new().changer()
        // 整合ParamsProcessor，完成初始化
        .params_processor_init(|| Box::new(CombinedParamsProcessor::new()
            .add("/user", UserParamsProcessor)
            .add("/blog", BlogParamsProcessor)))
        // 需要`island`feature
        .island_manager_init(|| IslandManager::new().register()
            // 註冊一個名為 "Counter" 的 Island，並使用預設 Island佔位
            .add_id("Counter")
            // 註冊一個名為 "Counter1" 的 Island，，並指定其渲染函數和預設pops
            .add("Counter1", Box::new(render_counter), None);
            .finish())
        .finish()
        .init();

    // 記錄 SSR 組件初始化完成
}

// 主函數，這應替換成Rust Web Framework 
fn main() {
    // 初始化 SSR 組件
    init_ssr_components();

    // 獲取渲染器
    let renderer = get_renderer();

    // 定義一個簡單的渲染函數
    let render_fn: Box<dyn FnOnce(&str) -> Result<String, String>> = Box::new(|props| {
        let json_props = serde_json::from_str::<serde_json::Value>(props).map_err(|e| e.to_string())?;
        let content = format!("Blog content with props: {}", json_props);
        let result = serde_json::json!({
            "html": content,
            "css": "",
            "head": "",
            "body": ""
        });
        Ok(result.to_string())
    });

    // 渲染頁面
    let path = "/blog/1";
    let params = HashMap::from([("id".to_string(), "1".to_string())]);
    let result = renderer.render(path, params, render_fn);

    match result {
        Ok((html, _cookies)) => println!("Rendered HTML: {}", html),
        Err(e) => eprintln!("Render error: {}", e),
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
let result = renderer.render(
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
    ,&CombinedIslandProcessor, // 如開啟了`island`才需要
);
```

## 後續工作
- 盡力在9月中前添加數個example
- 預期追加`i18n` feature

## 貢獻

我們歡迎社區貢獻！如果你有任何改進建議或發現了 bug，請開啟一個 issue 或提交一個 pull request。

## 許可

ssrkit 使用 MIT 許可證。

## 致謝

特別感謝 ssr-rs 專案的開發者，他的工作為 ssrkit 奠定了基礎。

## 相關項目

- [ssr-rs](https://github.com/Valerioageno/ssr-rs): ssrkit 的基礎 SSR 引擎s

## 更新日誌

請查看 [CHANGELOG.md](CHANGELOG.md) 檔案以了解最新的變更和版本資訊。

## 聯絡我們

如果你有任何問題或建議，可以透過以下方式聯絡我：

- GitHub Issues: [ssrkit issues](https://github.com/jeromeleong/ssrkit/issues)
- Email: jeromeleong1998@gmail.com

感謝你使用 ssrkit！我們期待看到你用它構建的出色應用。