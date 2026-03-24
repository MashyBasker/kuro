use anyhow::Result;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq)]
pub struct Project {
    pub root: PathBuf,
    pub source_dir: PathBuf,
    pub out_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub template_dir: PathBuf,
}

pub struct Templates {
    pub base: String,
    pub header: String,
    pub footer: String,
    pub page: String,
    pub post: String,
}

impl Project {
    pub fn new(root: PathBuf) -> Self {
        Self {
            source_dir: root.join("content"),
            out_dir: root.join("dist"),
            assets_dir: root.join("static"),
            template_dir: root.join("templates"),
            root,
        }
    }
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
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub date: Option<String>,
}
