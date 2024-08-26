use crate::config::SsrkitConfig;
use crate::Cache;
use chrono::{DateTime, Duration, Utc};
use nanoid::nanoid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;

pub struct Cookie {
    pub name: String,
    pub value: String,
    pub expires: Option<DateTime<Utc>>,
    pub max_age: Option<Duration>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

impl Cookie {
    pub fn new(name: String, value: String) -> Self {
        Self {
            name,
            value,
            expires: None,
            max_age: None,
            domain: None,
            path: None,
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    pub fn to_header_string(&self) -> String {
        let mut parts = vec![format!("{}={}", self.name, self.value)];

        if let Some(expires) = self.expires {
            parts.push(format!(
                "Expires={}",
                expires.format("%a, %d %b %Y %H:%M:%S GMT")
            ));
        }
        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age.num_seconds()));
        }
        if let Some(ref domain) = self.domain {
            parts.push(format!("Domain={}", domain));
        }
        if let Some(ref path) = self.path {
            parts.push(format!("Path={}", path));
        }
        if self.secure {
            parts.push("Secure".to_string());
        }
        if self.http_only {
            parts.push("HttpOnly".to_string());
        }
        if let Some(ref same_site) = self.same_site {
            parts.push(format!("SameSite={}", same_site));
        }

        parts.join("; ")
    }
}

pub struct CookieManager {
    cookies: HashMap<String, Cookie>,
}

impl Default for CookieManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CookieManager {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    pub fn add(&mut self, cookie: Cookie) {
        self.cookies.insert(cookie.name.clone(), cookie);
    }

    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.cookies.get(name)
    }

    pub fn delete(&mut self, name: &str) {
        self.cookies.remove(name);
    }

    pub fn update(&mut self, name: &str, value: String) {
        if let Some(cookie) = self.cookies.get_mut(name) {
            cookie.value = value;
        }
    }

    pub fn refresh(&mut self, name: &str) {
        if let Some(cookie) = self.cookies.get_mut(name) {
            cookie.expires = Some(Utc::now() + chrono::Duration::days(30));
        }
    }

    pub fn to_header_strings(&self) -> Vec<String> {
        self.cookies
            .values()
            .map(|c| c.to_header_string())
            .collect()
    }
}

pub struct Session {
    pub user_id: String,
    pub data: HashMap<String, String>,
    pub last_accessed: Instant,
}

impl Session {
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            data: HashMap::new(),
            last_accessed: Instant::now(),
        }
    }

    pub fn touch(&mut self) {
        self.last_accessed = Instant::now();
    }
}

pub struct SessionManager {
    sessions: HashMap<String, Session>,
    session_duration: std::time::Duration,
    config: Arc<SsrkitConfig>,
}

impl SessionManager {
    pub fn new(session_duration: std::time::Duration, config: Arc<SsrkitConfig>) -> Self {
        Self {
            sessions: HashMap::new(),
            session_duration,
            config,
        }
    }

    pub fn create_session(&mut self, user_id: String) -> String {
        let length = self.config.get_nanoid_length();
        let alphabet = self.config.get_nanoid_alphabet();
        let session_id = nanoid!(length, &alphabet);
        self.sessions
            .insert(session_id.clone(), Session::new(user_id));
        session_id
    }

    pub fn get_session(&mut self, session_id: &str) -> Option<&mut Session> {
        let duration = self.session_duration;
        self.sessions.get_mut(session_id).and_then(|session| {
            if session.last_accessed.elapsed() < duration {
                session.touch();
                Some(session)
            } else {
                None
            }
        })
    }

    pub fn remove_session(&mut self, session_id: &str) {
        self.sessions.remove(session_id);
    }

    pub fn cleanup_expired_sessions(&mut self) {
        self.sessions
            .retain(|_, session| session.last_accessed.elapsed() < self.session_duration);
    }
}

pub struct GlobalState {
    pub cache: Arc<Cache<String>>,
    pub cookie_manager: Arc<Mutex<CookieManager>>,
    pub session_manager: Arc<RwLock<SessionManager>>,
    pub config: Arc<SsrkitConfig>,
}

impl GlobalState {
    pub fn new(
        cache: Cache<String>,
        config: SsrkitConfig,
        session_duration: std::time::Duration,
    ) -> Self {
        let config = Arc::new(config);
        Self {
            cache: Arc::new(cache),
            cookie_manager: Arc::new(Mutex::new(CookieManager::new())),
            session_manager: Arc::new(RwLock::new(SessionManager::new(
                session_duration,
                config.clone(),
            ))),
            config,
        }
    }

    pub fn get_cache(&self) -> &Arc<Cache<String>> {
        &self.cache
    }

    pub fn get_cookie_manager(&self) -> &Arc<Mutex<CookieManager>> {
        &self.cookie_manager
    }

    pub fn get_session_manager(&self) -> &Arc<RwLock<SessionManager>> {
        &self.session_manager
    }

    pub fn get_config(&self) -> &Arc<SsrkitConfig> {
        &self.config
    }
}

// 全局静态变量
use std::sync::OnceLock;
static GLOBAL_STATE: OnceLock<RwLock<GlobalState>> = OnceLock::new();

pub fn init_global_state(
    cache: Cache<String>,
    config: SsrkitConfig,
    session_duration: std::time::Duration,
) {
    let _ = GLOBAL_STATE.set(RwLock::new(GlobalState::new(
        cache,
        config,
        session_duration,
    )));
}

pub fn get_global_state() -> &'static RwLock<GlobalState> {
    GLOBAL_STATE.get().expect("Global state not initialized")
}

pub fn set_global_state(new_state: GlobalState) -> Result<(), String> {
    match GLOBAL_STATE.get() {
        Some(lock) => {
            let mut state = lock.write().map_err(|e| e.to_string())?;
            *state = new_state;
            Ok(())
        }
        None => Err("Global state not initialized".to_string()),
    }
}
