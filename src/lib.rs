pub mod island;
pub mod params;
pub mod render;
pub mod template;
pub mod config;

// Re-export main types and traits
pub use island::{CombinedIslandProcessor, IslandManager, IslandProcessor, ProcessContext, get_or_render_island};
pub use params::{CombinedParamsProcessor, ParamsProcessor};
pub use render::{init_ssr, SsrRenderer, get_renderer};
pub use template::Template;
pub use config::SsrkitConfig;

// Re-export important types from serde_json that are commonly used
pub use serde_json::{Map, Value};

// Prelude module for convenient imports
pub mod prelude {
    pub use crate::island::{CombinedIslandProcessor, IslandManager, IslandProcessor, ProcessContext, get_or_render_island};
    pub use crate::params::{CombinedParamsProcessor, ParamsProcessor};
    pub use crate::render::{init_ssr, SsrRenderer, get_renderer};
    pub use crate::template::Template;
    pub use crate::config::SsrkitConfig;
    pub use serde_json::{Map, Value};
}