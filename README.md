# ssrkit

`ssrkit` 是一個輕量級的 Rust 庫，旨在簡化服務器端渲染（SSR）的實現過程。它提供了一個靈活的參數處理系統，可以輕鬆地與各種 Web 框架集成。

## 特性

- 輕量級和高效
- 靈活的參數處理系統
- 易於集成到現有的 Web 應用中
- 支持多種路由處理邏輯
- 線程安全

## 安裝

將以下內容添加到你的 `Cargo.toml` 文件中：

```toml
[dependencies]
ssrkit = { git = "http://git.leongfamily.net/jerome/ssrkit.git" }
```


## 使用方法

以下是一個基本的使用示例：

```rust
use ssrkit::{ParamsProcessor, CombinedParamsProcessor, get_or_init_renderer};
use std::collections::HashMap;
use serde_json::Value;

// 定義你的參數處理器
struct UserParamsProcessor;
impl ParamsProcessor for UserParamsProcessor {
    fn process(&self, path: &str, params: &HashMap<String, String>) -> serde_json::Map<String, Value> {
        // 實現你的用戶參數處理邏輯
    }
}

// 在你的請求處理函數中
fn handle_request(path: &str, params: HashMap<String, String>) -> Result<String, String> {
    let renderer = get_or_init_renderer(|| {
        CombinedParamsProcessor::new()
            .add("/user/", UserParamsProcessor)
            // 添加更多路由處理器...
    });

    renderer.render(path, params, |props| {
        // 實現你的渲染邏輯
        Ok("Rendered HTML".to_string())
    })
}
```

## API 參考

### `ParamsProcessor` trait

定義了參數處理的接口。

### `CombinedParamsProcessor`

允許組合多個 `ParamsProcessor` 實例，根據路徑前綴選擇適當的處理器。

### `get_or_init_renderer`

獲取或初始化一個全局的 `SsrRenderer` 實例。

### `SsrRenderer`

處理實際的渲染過程。