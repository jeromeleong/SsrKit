#[cfg(feature = "island")]
pub mod island;

pub mod cache;
pub mod config;
pub mod init;
pub mod params;
pub mod render;
pub mod state;
pub mod template;

// Re-export main types and traits
#[cfg(feature = "island")]
pub use island::{
    get_or_render_island, CombinedIslandProcessor, IslandManager, IslandProcessor, ProcessContext,
};

pub use cache::{init_cache, Cache};
pub use config::{get_global_config, set_global_config, SsrkitConfig};
pub use init::SsrInitializer;
pub use params::{CombinedParamsProcessor, ParamsProcessor};
pub use render::{get_renderer, SsrRenderer};
pub use state::{
    get_global_state, init_global_state, Cookie, CookieManager, GlobalState, Session,
    SessionManager,
};
pub use template::Template;

#[cfg(feature = "island")]
pub use ssrkit_macros::island_handle;
pub use ssrkit_macros::params_handle;

// Re-export important types from serde_json that are commonly used
pub use serde_json::{Map, Value};

// Prelude module for convenient imports
pub mod prelude {
    #[cfg(feature = "island")]
    pub use crate::island::{
        get_or_render_island, CombinedIslandProcessor, IslandManager, IslandProcessor,
        ProcessContext,
    };

    pub use crate::cache::{init_cache, Cache};
    pub use crate::config::{get_global_config, set_global_config, SsrkitConfig};
    pub use crate::init::SsrInitializer;
    pub use crate::params::{CombinedParamsProcessor, ParamsProcessor};
    pub use crate::render::{get_renderer, SsrRenderer};
    pub use crate::state::{
        get_global_state, init_global_state, Cookie, CookieManager, GlobalState, Session,
        SessionManager,
    };
    pub use crate::template::Template;

    #[cfg(feature = "island")]
    pub use ssrkit_macros::island_handle;
    pub use ssrkit_macros::params_handle;

    pub use serde_json::{Map, Value};
}
