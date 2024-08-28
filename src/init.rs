use crate::cache::Cache;
use crate::config::{get_global_config, set_global_config, SsrkitConfig};
use crate::params::ParamsProcessor;
use crate::state::init_global_state;
use crate::template::{init_template_cache, Template};
use crate::{CombinedParamsProcessor, SsrRenderer};
use std::sync::Arc;
use std::sync::OnceLock;

#[cfg(feature = "island")]
use crate::island::{init_island_cache, IslandManager};
#[cfg(feature = "island")]
use regex::Regex;

pub static RENDERER: OnceLock<SsrRenderer> = OnceLock::new();
static TEMPLATE: OnceLock<Arc<Template>> = OnceLock::new();
// Global static variables
#[cfg(feature = "island")]
pub static ISLAND_REGEX: OnceLock<Regex> = OnceLock::new();
#[cfg(feature = "island")]
static ISLAND_MANAGER: OnceLock<Arc<IslandManager>> = OnceLock::new();

pub struct SsrInitializer {
    params_processor_init: Box<dyn FnOnce() -> Box<dyn ParamsProcessor>>,
    template_init: Box<dyn FnOnce() -> Template>,
    config: Option<SsrkitConfig>,
    #[cfg(feature = "island")]
    island_manager_init: Box<dyn FnOnce() -> IslandManager>,
}

impl Default for SsrInitializer {
    fn default() -> Self {
        Self::new()
    }
}
impl SsrInitializer {
    pub fn new() -> Self {
        Self {
            params_processor_init: Box::new(|| Box::new(CombinedParamsProcessor::new())),
            template_init: Box::new(Template::new),
            config: Some(SsrkitConfig::default()),
            #[cfg(feature = "island")]
            island_manager_init: Box::new(IslandManager::new),
        }
    }

    pub fn changer() -> SsrInitializerChanger {
        SsrInitializerChanger {
            initializer: Self::new(),
        }
    }

    pub fn init(self) {
        let config = self.config.unwrap_or_else(|| get_global_config().clone());

        // 設置全局配置
        set_global_config(config.clone());

        // 初始化全局配置
        crate::cache::init_cache(&config);

        // 初始化 GlobalState
        let cache = Cache::new(|config| config.get_global_state_cache_size());
        let session_duration = config.get_global_state_session_duration();
        init_global_state(cache, config.clone(), session_duration);

        #[cfg(feature = "island")]
        // 初始化正則表達式
        ISLAND_REGEX.get_or_init(|| {
            Regex::new(r#"<div data-island="([^"]+)"(?: data-props='([^']*)')?></div>"#).unwrap()
        });

        #[cfg(feature = "island")]
        {
            // 初始化 IslandManager
            let island_manager = (self.island_manager_init)();
            init_island_cache();
            ISLAND_MANAGER.get_or_init(|| Arc::new(island_manager));
        }

        // 初始化 Template
        let template = (self.template_init)();
        init_template_cache();
        TEMPLATE.get_or_init(|| Arc::new(template));

        // 初始化 Renderer
        RENDERER.get_or_init(|| {
            SsrRenderer::new(
                (self.params_processor_init)(),
                #[cfg(feature = "island")]
                ISLAND_MANAGER.get().unwrap().clone(),
                TEMPLATE.get().unwrap().clone(),
            )
        });
    }
}

pub struct SsrInitializerChanger {
    initializer: SsrInitializer,
}

impl SsrInitializerChanger {
    pub fn params_processor_init(
        mut self,
        params_processor_init: impl FnOnce() -> Box<dyn ParamsProcessor> + 'static,
    ) -> Self {
        self.initializer.params_processor_init = Box::new(params_processor_init);
        self
    }

    pub fn template_init(mut self, template_init: impl FnOnce() -> Template + 'static) -> Self {
        self.initializer.template_init = Box::new(template_init);
        self
    }

    pub fn config(mut self, config: SsrkitConfig) -> Self {
        self.initializer.config = Some(config);
        self
    }

    #[cfg(feature = "island")]
    pub fn island_manager_init(
        mut self,
        island_manager_init: impl FnOnce() -> IslandManager + 'static,
    ) -> Self {
        self.initializer.island_manager_init = Box::new(island_manager_init);
        self
    }

    pub fn finish(self) -> SsrInitializer {
        self.initializer
    }
}
