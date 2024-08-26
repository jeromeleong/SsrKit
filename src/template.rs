use crate::Cache;
use serde_json::Value;
use std::sync::OnceLock;

#[cfg(feature = "island")]
use std::collections::HashSet;

static TEMPLATE_CACHE: OnceLock<Cache<String>> = OnceLock::new();

pub struct Template;

impl Default for Template {
    fn default() -> Self {
        Self::new()
    }
}

impl Template {
    pub fn new() -> Self {
        Self
    }

    #[cfg(feature = "island")]
    pub fn render(&self, content: &Value, islands: &Value) -> Result<String, String> {
        let cache_key = format!("{:?}:{:?}", content, islands);

        // Try to get from cache
        if let Some(cached_html) = TEMPLATE_CACHE.get().unwrap().get(&cache_key) {
            return Ok(cached_html);
        }

        let html = content["html"]
            .as_str()
            .ok_or("Missing 'html' in content")?;
        let css = content["css"].as_str().unwrap_or("");
        let head_extra = content["head"].as_str().unwrap_or("");
        let body_extra = content["body"].as_str().unwrap_or("");

        let island_scripts = self.generate_island_scripts(islands);

        let rendered_html =
            if html.trim().starts_with("<!DOCTYPE html>") || html.trim().starts_with("<html>") {
                // If it's a complete HTML, we just need to insert extra content
                let mut rendered_html = html.to_string();

                // Insert extra head content before </head>
                if let Some(head_end) = rendered_html.find("</head>") {
                    rendered_html.insert_str(
                        head_end,
                        &format!("{}<style>{}</style>{}", head_extra, css, island_scripts),
                    );
                }

                // Insert extra body content after <body>
                if let Some(body_start) = rendered_html.find("<body>") {
                    rendered_html.insert_str(body_start + 6, body_extra);
                }

                rendered_html
            } else {
                // If it's not a complete HTML, use our template
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

        // Store result in cache
        TEMPLATE_CACHE
            .get()
            .unwrap()
            .insert(&cache_key, final_html.clone());

        Ok(final_html)
    }

    #[cfg(not(feature = "island"))]
    pub fn render(&self, content: &Value) -> Result<String, String> {
        let cache_key = format!("{:?}", content);

        // Try to get from cache
        if let Some(cached_html) = TEMPLATE_CACHE.get().unwrap().get(&cache_key) {
            return Ok(cached_html);
        }

        let html = content["html"]
            .as_str()
            .ok_or("Missing 'html' in content")?;
        let css = content["css"].as_str().unwrap_or("");
        let head_extra = content["head"].as_str().unwrap_or("");
        let body_extra = content["body"].as_str().unwrap_or("");

        let rendered_html =
            if html.trim().starts_with("<!DOCTYPE html>") || html.trim().starts_with("<html>") {
                // If it's a complete HTML, we just need to insert extra content
                let mut rendered_html = html.to_string();

                // Insert extra head content before </head>
                if let Some(head_end) = rendered_html.find("</head>") {
                    rendered_html
                        .insert_str(head_end, &format!("{}<style>{}</style>", head_extra, css));
                }

                // Insert extra body content after <body>
                if let Some(body_start) = rendered_html.find("<body>") {
                    rendered_html.insert_str(body_start + 6, body_extra);
                }

                rendered_html
            } else {
                // If it's not a complete HTML, use our template
                indoc::formatdoc! {r#"
                <!DOCTYPE html>
                <html>
                <head>
                    {head_extra}
                    <style>{css}</style>
                </head>
                <body>
                    {body_extra}
                    {html}
                </body>
                </html>
            "#}
            };

        // Store result in cache
        TEMPLATE_CACHE
            .get()
            .unwrap()
            .insert(&cache_key, rendered_html.clone());

        Ok(rendered_html)
    }

    #[cfg(feature = "island")]
    fn replace_island_placeholders(&self, html: &mut String, islands: &Value) {
        if let Some(island_instances) = islands.as_object() {
            for (name, instance) in island_instances {
                if let (Some(id), Some(island_html)) =
                    (instance["id"].as_str(), instance["html"].as_str())
                {
                    let placeholder =
                        format!(r#"<div data-island="{}" data-name="{}"></div>"#, id, name);
                    *html = html.replace(&placeholder, island_html);
                }
            }
        }
    }

    #[cfg(feature = "island")]
    fn generate_island_scripts(&self, islands: &Value) -> String {
        let unique_islands: HashSet<&str> = islands
            .as_object()
            .map(|obj| obj.values().filter_map(|v| v["id"].as_str()).collect())
            .unwrap_or_default();

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

pub fn init_template_cache() {
    TEMPLATE_CACHE.get_or_init(|| Cache::new(|config| config.get_template_cache_size()));
}

pub fn render_template<F>(key: &str, render_fn: F) -> String
where
    F: FnOnce() -> String,
{
    TEMPLATE_CACHE
        .get()
        .expect("Template cache not initialized")
        .get_or_insert(key, render_fn)
}
