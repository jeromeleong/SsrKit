pub mod cache;
pub mod config;
pub mod island;
pub mod params;
pub mod render;
pub mod state;
pub mod template;

// Re-export main types and traits
pub use cache::{init_cache, Cache};
pub use config::SsrkitConfig;
pub use island::{get_or_render_island, IslandManager, IslandProcessor, ProcessContext};
pub use params::{CombinedParamsProcessor, ParamsProcessor};
pub use render::{get_renderer, init_ssr, SsrRenderer};
pub use template::Template;

// Re-export important types from serde_json that are commonly used
pub use serde_json::{Map, Value};

// Prelude module for convenient imports
pub mod prelude {
    pub use crate::cache::{init_cache, Cache};
    pub use crate::config::SsrkitConfig;
    pub use crate::island::{get_or_render_island, IslandManager, IslandProcessor, ProcessContext};
    pub use crate::params::{CombinedParamsProcessor, ParamsProcessor};
    pub use crate::render::{get_renderer, init_ssr, SsrRenderer};
    pub use crate::template::Template;
    pub use serde_json::{Map, Value};
}
