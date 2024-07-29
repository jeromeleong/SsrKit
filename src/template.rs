use serde_json::Value;
use std::collections::HashSet;
use std::sync::OnceLock;
use std::sync::Mutex;
use lru::LruCache;
use std::num::NonZeroUsize;

static TEMPLATE_CACHE: OnceLock<Mutex<LruCache<String, String>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<LruCache<String, String>> {
    TEMPLATE_CACHE.get_or_init(|| {
        Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))
    })
}

pub struct Template;

impl Template {
    pub fn new() -> Self {
        Self
    }

    pub fn render(&self, content: &Value, islands: &Value) -> Result<String, String> {
        let cache_key = format!("{:?}:{:?}", content, islands);
        
        // 嘗試從緩存中獲取
        if let Some(cached_html) = get_cache().lock().unwrap().get(&cache_key) {
            return Ok(cached_html.clone());
        }

        let html = content["html"].as_str().ok_or("Missing 'html' in content")?;
        let css = content["css"].as_str().unwrap_or("");
        let head_extra = content["head"].as_str().unwrap_or("");
        let body_extra = content["body"].as_str().unwrap_or("");

        let island_scripts = self.generate_island_scripts(islands);

        let rendered_html = if html.trim().starts_with("<!DOCTYPE html>") || html.trim().starts_with("<html>") {
            // 如果是完整的 HTML，我們只需要插入額外的內容
            let mut rendered_html = html.to_string();

            // 在 </head> 之前插入額外的頭部內容
            if let Some(head_end) = rendered_html.find("</head>") {
                rendered_html.insert_str(head_end, &format!("{}<style>{}</style>{}", head_extra, css, island_scripts));
            }

            // 在 <body> 之後插入額外的身體內容
            if let Some(body_start) = rendered_html.find("<body>") {
                rendered_html.insert_str(body_start + 6, body_extra);
            }

            rendered_html
        } else {
            // 如果不是完整的 HTML，使用我們的模板
            indoc::formatdoc! {r#"
                <!DOCTYPE html>
                <html>
                <head>
                    {head_extra}
                    <style>{css}</style>
                    {island_scripts}
                </head>
                <body>
                    {body_extra}
                    {html}
                </body>
                </html>
            "#}
        };

        let mut final_html = rendered_html;
        self.replace_island_placeholders(&mut final_html, islands);

        // 將結果存入緩存
        get_cache().lock().unwrap().put(cache_key, final_html.clone());

        Ok(final_html)
    }

    fn replace_island_placeholders(&self, html: &mut String, islands: &Value) {
        if let Some(island_instances) = islands.as_object() {
            for (name, instance) in island_instances {
                if let (Some(id), Some(island_html)) = (instance["id"].as_str(), instance["html"].as_str()) {
                    let placeholder = format!(r#"<div data-island="{}" data-name="{}"></div>"#, id, name);
                    *html = html.replace(&placeholder, island_html);
                }
            }
        }
    }

    fn generate_island_scripts(&self, islands: &Value) -> String {
        let unique_islands: HashSet<&str> = islands
            .as_object()
            .map(|obj| {
                obj.values()
                    .filter_map(|v| v["id"].as_str())
                    .collect()
            })
            .unwrap_or_else(HashSet::new);

        unique_islands
            .into_iter()
            .map(|id| {
                format!(
                    r#"<script type="module" src="/islands/{}.js" async></script>"#,
                    id.to_lowercase()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}