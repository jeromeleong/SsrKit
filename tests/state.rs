use ssrkit::prelude::*;
use std::time::Duration;

#[test]
fn test_session_manager() {
    // 測試會話管理器
    let config = SsrkitConfig::default();
    let cache = Cache::new(|config| config.get_global_state_cache_size());
    let session_duration = Duration::from_secs(3600);
    init_global_state(cache, config, session_duration);

    let binding = get_global_state().read().unwrap();
    let session_manager = binding.get_session_manager();
    let user_id = "test_user".to_string();
    let session_id = session_manager
        .write()
        .unwrap()
        .create_session(user_id.clone());

    let mut binding = session_manager.write().unwrap();
    let session = binding.get_session(&session_id);
    assert!(session.is_some());
    assert_eq!(session.unwrap().user_id, user_id);
}
