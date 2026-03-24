use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Project {
    pub root: PathBuf,
    pub source_dir: PathBuf,
    pub out_dir: PathBuf,
    pub assets_dir: PathBuf,
    pub template_dir: PathBuf,
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

#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub date: Option<String>,
}
