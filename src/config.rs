use std::num::NonZeroUsize;

#[derive(Clone)]
pub struct SsrkitConfig {
    pub island_cache_size: NonZeroUsize,
    pub template_cache_size: NonZeroUsize,
}

impl Default for SsrkitConfig {
    fn default() -> Self {
        Self {
            island_cache_size: NonZeroUsize::new(100).unwrap(),
            template_cache_size: NonZeroUsize::new(100).unwrap(),
        }
    }
}
