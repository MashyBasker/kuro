use anyhow::Result;
use serde::Deserialize;
use std::{fs, path::Path};

pub struct Templates {
    pub base: String,
    pub header: String,
    pub footer: String,
    pub page: String,
    pub post: String,
    pub writings: String,
}

impl Templates {
    pub fn load(path: &Path) -> Result<Self> {
        let t = path.join("templates");

        Ok(Self {
            base: fs::read_to_string(t.join("base.html"))?,
            header: fs::read_to_string(t.join("header.html"))?,
            footer: fs::read_to_string(t.join("footer.html"))?,
            page: fs::read_to_string(t.join("page.html"))?,
            post: fs::read_to_string(t.join("post.html"))?,
            writings: fs::read_to_string(t.join("writings.html"))?,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub date: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct PostMeta {
    pub title: String,
    pub date: Option<String>,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct FooterLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct SitePaths {
    pub label: String,
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub footer_links: Vec<FooterLink>,
    #[serde(default)]
    pub site_paths: Vec<SitePaths>,
}
