use anyhow::Result;
use gray_matter::{Matter, Pod, engine::YAML};
use std::{fs, path::Path};

use crate::types::{Frontmatter, PostMeta, Templates};

pub fn show_help() {
    println!("\n  Kuro - Static Site Generator");
    println!("");
    println!("  Usage:");
    println!("    kuro init [path]   Create a new site (default: current directory)");
    println!("    kuro build [path]  Build the site (default: current directory)");
    println!("    kuro -h, --help    Show this help");
    println!("");
}

pub fn copy_dir(src: &Path, dest: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dest.join(entry.file_name());

        if path.is_dir() {
            fs::create_dir_all(&target)?;
            copy_dir(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

pub fn md_to_html(input: &str) -> String {
    use comrak::{Options, markdown_to_html};

    let mut options = Options::default();

    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.strikethrough = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;

    options.parse.smart = true;

    markdown_to_html(input, &options)
}

pub fn parse_content(input: &str) -> Result<(Option<Frontmatter>, String)> {
    let matter = Matter::<YAML>::new();

    let result = matter.parse(input)?;

    let data = result
        .data
        .and_then(|d: Pod| d.deserialize::<Frontmatter>().ok());

    Ok((data, result.content))
}

pub fn render_page(input: &str, templates: &Templates) -> anyhow::Result<String> {
    let (fm, content) = crate::utils::parse_content(input)?;

    let html_content = md_to_html(&content);

    let title = fm.as_ref().map(|f| f.title.as_str()).unwrap_or("Untitled");

    // page.html only needs content
    let body = templates.page.replace("{content}", &html_content);

    // final composition
    let full = templates
        .base
        .replace("{title}", title)
        .replace("{header}", &templates.header)
        .replace("{content}", &body)
        .replace("{footer}", &templates.footer);

    Ok(full)
}

pub fn render_writings(posts: &[PostMeta], templates: &Templates) -> anyhow::Result<String> {
    let items: String = posts
        .iter()
        .map(|p| {
            let date = p
                .date
                .as_deref()
                .map(|d| format!(" <span class=\"date\">{}</span>", d))
                .unwrap_or_default();
            format!("<li><a href=\"{}\">{}</a>{}</li>", p.url, p.title, date)
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    let body = templates.writings.replace("{posts}", &items);

    let full = templates
        .base
        .replace("{title}", "Writings")
        .replace("{header}", &templates.header)
        .replace("{content}", &body)
        .replace("{footer}", &templates.footer);

    Ok(full)
}

pub fn render_post(input: &str, templates: &Templates) -> anyhow::Result<String> {
    let (fm, content) = crate::utils::parse_content(input)?;

    let html_content = md_to_html(&content);

    let title = fm.as_ref().map(|f| f.title.as_str()).unwrap_or("Untitled");

    let date_html = fm
        .as_ref()
        .and_then(|f| f.date.as_ref())
        .map(|d| format!(r#"<p class="date">{}</p>"#, d))
        .unwrap_or_default();

    // post.html includes title + date + content
    let body = templates
        .post
        .replace("{title}", title)
        .replace("{date}", &date_html)
        .replace("{content}", &html_content);

    let full = templates
        .base
        .replace("{title}", title)
        .replace("{header}", &templates.header)
        .replace("{content}", &body)
        .replace("{footer}", &templates.footer);

    Ok(full)
}
