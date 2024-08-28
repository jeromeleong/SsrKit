use ssrkit::prelude::*;
use std::num::NonZeroUsize;
use std::time::Duration;

#[test]
fn test_config_change() {
    // 測試配置構建器的功能
    let config = SsrkitConfig::change()
        .nanoid_length(10)
        .nanoid_alphabet("abc".chars().collect())
        .global_state_cache_size(NonZeroUsize::new(500).unwrap())
        .global_state_session_duration(Duration::from_secs(1800))
        .template_cache_size(NonZeroUsize::new(50).unwrap())
        .finish();

    assert_eq!(config.get_nanoid_length(), 10);
    assert_eq!(
        config.get_nanoid_alphabet(),
        "abc".chars().collect::<Vec<_>>()
    );
    assert_eq!(config.get_global_state_cache_size().get(), 500);
    assert_eq!(
        config.get_global_state_session_duration(),
        Duration::from_secs(1800)
    );
    assert_eq!(config.get_template_cache_size().get(), 50);
}
