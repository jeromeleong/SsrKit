use serde_json::Value;
use std::collections::HashSet;

pub struct Template {
    base_html: String,
}

impl Template {
    pub fn new(base_html: &str) -> Self {
        Self {
            base_html: base_html.to_owned(),
        }
    }

    pub fn render(&self, content: &Value, islands: &Value) -> Result<String, String> {
        let static_html = content["html"]
            .as_str()
            .ok_or("Missing 'html' in content")?;
        let css = content["css"].as_str().unwrap_or("");
        let head = content["head"].as_str().unwrap_or("");

        let island_scripts = self.generate_island_scripts(islands);

        let mut rendered_html = self
            .base_html
            .replace(
                "%ssrkit.head%",
                &format!("{}<style>{}</style>{}", head, css, island_scripts),
            )
            .replace("%ssrkit.body%", static_html);

        // Replace Island placeholders with custom rendered content
        if let Some(island_instances) = islands.as_object() {
            for (name, instance) in island_instances {
                if let (Some(id), Some(html)) = (instance["id"].as_str(), instance["html"].as_str()) {
                    let placeholder = format!(r#"<div data-island="{}" data-name="{}"></div>"#, id, name);
                    rendered_html = rendered_html.replace(&placeholder, html);
                }
            }
        }

        Ok(rendered_html)
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