use ssrkit::prelude::*;
use std::num::NonZeroUsize;

#[test]
fn test_cache_insert_and_get() {
    // 測試緩存的插入和獲取功能
    let cache = Cache::new(|_config| NonZeroUsize::new(1).unwrap());
    let key = "test_key";
    let value = "test_value";
    cache.insert(key, value);
    assert_eq!(cache.get(key), Some(value));
}

#[test]
fn test_cache_get_or_insert() {
    // 測試緩存的 get_or_insert 功能
    let cache = Cache::new(|_config| NonZeroUsize::new(1).unwrap());
    let key = "test_key";
    let value = "test_value";
    let result = cache.get_or_insert(key, || value);
    assert_eq!(result, value);
    assert_eq!(cache.get(key), Some(value));
}
