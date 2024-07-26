use serde_json::Value;

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

        Ok(self
            .base_html
            .replace(
                "%ssrkit.head%",
                &format!("{}<style>{}</style>{}", head, css, island_scripts),
            )
            .replace("%ssrkit.body%", static_html))
    }

    fn generate_island_scripts(&self, islands: &Value) -> String {
        islands
            .as_object()
            .map(|obj| {
                obj.keys()
                    .map(|id| {
                        format!(
                            r#"<script type="module" src="/islands/{}.js" async></script>"#,
                            id.to_lowercase()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default()
    }
}
