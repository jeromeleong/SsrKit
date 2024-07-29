pub mod island;
pub mod params;
pub mod render;
pub mod template;

// Re-export main types and traits
pub use island::{CombinedIslandProcessor, IslandManager, IslandProcessor};
pub use params::{CombinedParamsProcessor, ParamsProcessor};
pub use render::{init_renderer, SsrRenderer};
pub use template::Template;

// Re-export important types from serde_json that are commonly used
pub use serde_json::{Map, Value};

// You might want to add a prelude module for convenient imports
pub mod prelude {
    pub use crate::island::{CombinedIslandProcessor, IslandManager, IslandProcessor, ProcessContext};
    pub use crate::params::{CombinedParamsProcessor, ParamsProcessor};
    pub use crate::render::{init_renderer, SsrRenderer};
    pub use crate::template::Template;
    pub use serde_json::{Map, Value};
}

// If you have any error types, you might want to re-export them here
// pub use crate::error::SsrKitError;

// If you have any configuration structs, you might want to re-export them here
// pub use crate::config::SsrKitConfig;

// You might want to add a convenience function for creating a new SsrRenderer
pub fn create_ssr_renderer<P, F>(
    params_processor: F,
    island_manager: std::sync::Arc<IslandManager>,
    template: std::sync::Arc<Template>,
) -> &'static SsrRenderer
where
    P: ParamsProcessor + 'static,
    F: FnOnce() -> P,
{
    init_renderer(params_processor, island_manager, template)
}
