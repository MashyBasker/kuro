use anyhow::Result;

use crate::{
    core::CARD_HTML,
    types::{Config, PostMeta, Templates},
    utils::{md_to_html, parse_content},
};

pub struct SiteRenderer {
    templates: Templates,
    footer_links: String,
    nav_bar: String,
}

impl SiteRenderer {
    pub fn new(templates: Templates, yaml_config: &str) -> Result<Self> {
        let config: Config = serde_yaml::from_str(yaml_config)?;

        let footer_links = config
            .footer_links
            .iter()
            .map(|l| format!("<a href=\"{}\">{}</a>", l.url, l.label))
            .collect::<Vec<_>>()
            .join("\t");

        let nav_bar = config
            .site_paths
            .iter()
            .map(|sp| format!("<a href=\"{}\">{}</a>", sp.path, sp.label))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(Self { templates, footer_links, nav_bar })
    }

    pub fn render_page(&self, input: &str) -> Result<String> {
        let (fm, content) = parse_content(input)?;
        let html_content = md_to_html(&content);
        let title = fm.as_ref().map(|f| f.title.as_str()).unwrap_or("Untitled");

        let body = self.templates.page.replace("{content}", &html_content);
        let footer = self.templates.footer.replace("{links}", &self.footer_links);
        let header = self.templates.header.replace("{links}", &self.nav_bar);

        Ok(self.templates.base
            .replace("{title}", title)
            .replace("{header}", &header)
            .replace("{content}", &body)
            .replace("{footer}", &footer))
    }

    pub fn render_post(&self, input: &str) -> Result<String> {
        let (fm, content) = parse_content(input)?;
        let html_content = md_to_html(&content);
        let title = fm.as_ref().map(|f| f.title.as_str()).unwrap_or("Untitled");

        let footer = self.templates.footer.replace("{links}", &self.footer_links);
        let header = self.templates.header.replace("{links}", &self.nav_bar);

        let date_html = fm
            .as_ref()
            .and_then(|f| f.date.as_ref())
            .map(|d| format!(r#"<p class="date">{}</p>"#, d))
            .unwrap_or_default();

        let body = self.templates.post
            .replace("{title}", title)
            .replace("{date}", &date_html)
            .replace("{content}", &html_content);

        Ok(self.templates.base
            .replace("{title}", title)
            .replace("{header}", &header)
            .replace("{content}", &body)
            .replace("{footer}", &footer))
    }

    pub fn render_writings(&self, posts: &[PostMeta]) -> Result<String> {
        let items: String = posts
            .iter()
            .map(|p| {
                let date = p
                    .date
                    .as_deref()
                    .map(|d| format!("\n      <span class=\"post-card-date\">{}</span>", d))
                    .unwrap_or_default();
                CARD_HTML
                    .replace("{url}", &p.url)
                    .replace("{title}", &p.title)
                    .replace("{date}", &date)
            })
            .collect::<Vec<_>>()
            .join("\n    ");

        let footer = self.templates.footer.replace("{links}", &self.footer_links);
        let header = self.templates.header.replace("{links}", &self.nav_bar);
        let body = self.templates.writings.replace("{posts}", &items);

        Ok(self.templates.base
            .replace("{title}", "Writings")
            .replace("{header}", &header)
            .replace("{content}", &body)
            .replace("{footer}", &footer))
    }
}
