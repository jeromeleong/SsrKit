use std::num::NonZeroUsize;
use std::sync::OnceLock;
use std::time::Duration;

pub struct SsrkitConfig {
    pub island_cache_size: Option<NonZeroUsize>,
    pub template_cache_size: Option<NonZeroUsize>,
    pub nanoid_length: Option<usize>,
    pub nanoid_alphabet: Option<Vec<char>>,
    pub global_state_cache_size: Option<NonZeroUsize>,
    pub global_state_session_duration: Option<Duration>,
}

impl SsrkitConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> SsrkitConfigBuilder {
        SsrkitConfigBuilder::new()
    }

    pub fn get_island_cache_size(&self) -> NonZeroUsize {
        self.island_cache_size
            .unwrap_or(NonZeroUsize::new(100).unwrap())
    }

    pub fn get_template_cache_size(&self) -> NonZeroUsize {
        self.template_cache_size
            .unwrap_or(NonZeroUsize::new(100).unwrap())
    }

    pub fn get_nanoid_length(&self) -> usize {
        self.nanoid_length.unwrap_or(21)
    }

    pub fn get_nanoid_alphabet(&self) -> Vec<char> {
        self.nanoid_alphabet.clone().unwrap_or_else(|| {
            "ABCDEFGHJKMNPQRSTUVWXYZ\
             abcdefghjkmnpqrstuvwxyz\
             23456789"
                .chars()
                .collect()
        })
    }

    pub fn get_global_state_cache_size(&self) -> NonZeroUsize {
        self.global_state_cache_size
            .unwrap_or(NonZeroUsize::new(1000).unwrap())
    }

    pub fn get_global_state_session_duration(&self) -> Duration {
        self.global_state_session_duration.unwrap_or(Duration::from_secs(3600))
    }
}

impl Default for SsrkitConfig {
    fn default() -> Self {
        Self {
            island_cache_size: Some(NonZeroUsize::new(100).unwrap()),
            template_cache_size: Some(NonZeroUsize::new(100).unwrap()),
            nanoid_length: Some(21),
            nanoid_alphabet: Some(
                "ABCDEFGHJKMNPQRSTUVWXYZ\
                 abcdefghjkmnpqrstuvwxyz\
                 23456789"
                    .chars()
                    .collect(),
            ),
            global_state_cache_size: Some(NonZeroUsize::new(1000).unwrap()),
            global_state_session_duration: Some(Duration::from_secs(3600)),
        }
    }
}

impl Clone for SsrkitConfig {
    fn clone(&self) -> Self {
        Self {
            island_cache_size: self.island_cache_size,
            template_cache_size: self.template_cache_size,
            nanoid_length: self.nanoid_length,
            nanoid_alphabet: self.nanoid_alphabet.clone(),
            global_state_cache_size: self.global_state_cache_size,
            global_state_session_duration: self.global_state_session_duration,
        }
    }
}

pub struct SsrkitConfigBuilder {
    island_cache_size: Option<NonZeroUsize>,
    template_cache_size: Option<NonZeroUsize>,
    nanoid_length: Option<usize>,
    nanoid_alphabet: Option<Vec<char>>,
    global_state_cache_size: Option<NonZeroUsize>,
    global_state_session_duration: Option<Duration>,
}

impl SsrkitConfigBuilder {
    pub fn new() -> Self {
        Self {
            island_cache_size: None,
            template_cache_size: None,
            nanoid_length: None,
            nanoid_alphabet: None,
            global_state_cache_size: None,
            global_state_session_duration: None,
        }
    }

    pub fn island_cache_size(mut self, size: NonZeroUsize) -> Self {
        self.island_cache_size = Some(size);
        self
    }

    pub fn template_cache_size(mut self, size: NonZeroUsize) -> Self {
        self.template_cache_size = Some(size);
        self
    }

    pub fn nanoid_length(mut self, length: usize) -> Self {
        self.nanoid_length = Some(length);
        self
    }

    pub fn nanoid_alphabet(mut self, alphabet: Vec<char>) -> Self {
        self.nanoid_alphabet = Some(alphabet);
        self
    }

    pub fn global_state_cache_size(mut self, size: NonZeroUsize) -> Self {
        self.global_state_cache_size = Some(size);
        self
    }

    pub fn global_state_session_duration(mut self, duration: Duration) -> Self {
        self.global_state_session_duration = Some(duration);
        self
    }

    pub fn build(self) -> SsrkitConfig {
        SsrkitConfig {
            island_cache_size: self.island_cache_size,
            template_cache_size: self.template_cache_size,
            nanoid_length: self.nanoid_length,
            nanoid_alphabet: self.nanoid_alphabet,
            global_state_cache_size: self.global_state_cache_size,
            global_state_session_duration: self.global_state_session_duration,
        }
    }
}

impl Default for SsrkitConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

static GLOBAL_CONFIG: OnceLock<SsrkitConfig> = OnceLock::new();

pub fn set_global_config(config: SsrkitConfig) {
    let _ = GLOBAL_CONFIG.set(config);
}

pub fn get_global_config() -> &'static SsrkitConfig {
    GLOBAL_CONFIG.get().expect("Global config not initialized")
}