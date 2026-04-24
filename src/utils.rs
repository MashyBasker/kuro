use anyhow::Result;
use gray_matter::{Matter, Pod, engine::YAML};
use std::{fs, path::Path};

use crate::types::{Config, Frontmatter, PostMeta, Templates};

pub fn show_help() {
    println!("\n kuro {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!(" USAGE:");
    println!("     kuro <COMMAND> [OPTIONS]");
    println!();
    println!(" COMMANDS:");
    println!("     init [PATH]       Create a new site (default: current directory)");
    println!("     build [PATH]      Build the site (default: current directory)");
    println!("     serve [PATH]      Serve the site locally (default: current directory)");
    println!("     new <NAME>        Create a new page");
    println!();
    println!(" OPTIONS:");
    println!("     --post            Create as a blog post (used with `new`)");
    println!("     -h, --help        Print this help message");
    println!();
    println!(" EXAMPLES:");
    println!("     kuro init                  Create a new site in the current directory");
    println!("     kuro init my-site          Create a new site in ./my-site");
    println!("     kuro build                 Build the site");
    println!("     kuro new about             Create a new page called 'about'");
    println!("     kuro new my-post --post    Create a new blog post\n");
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

pub fn build_header_html(extra_pages: &[(String, String)]) -> String {
    let mut links = String::from("    <a href=\"/\">Home</a>");
    for (title, url) in extra_pages {
        links.push_str(&format!("\n    <a href=\"{}\">{}</a>", url, title));
    }
    links.push_str("\n    <a href=\"/writings/\">Writings</a>");
    format!("\n<header>\n  <nav>\n{}\n  </nav>\n</header>\n", links)
}

pub fn update_footer(yaml_config: &str) -> String {
    let config: Config = serde_yaml::from_str(yaml_config).unwrap();
    config
        .footer_links
        .iter()
        .map(|l| format!("<a href=\"{}\">{}</a>", l.url, l.label))
        .collect::<Vec<_>>()
        .join("\t")
}

pub fn get_site_path(yaml_config: &str) -> String {
    let config: Config = serde_yaml::from_str(yaml_config).unwrap();
    config
        .site_paths
        .iter()
        .map(|sp| format!("<a href=\"{}\">{}</a>", sp.path, sp.label))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render_page(input: &str, templates: &Templates, yaml_config: &str) -> anyhow::Result<String> {
    let (fm, content) = crate::utils::parse_content(input)?;

    let html_content = md_to_html(&content);

    let title = fm.as_ref().map(|f| f.title.as_str()).unwrap_or("Untitled");

    // page.html only needs content
    let body = templates.page.replace("{content}", &html_content);

    let footer_links = update_footer(yaml_config);
    let footer_content = templates.footer.replace("{links}", &footer_links);

    let nav_bar = get_site_path(yaml_config);
    let nvbar_contents = templates.header.replace("{links}", &nav_bar);

    // final composition
    let full = templates
        .base
        .replace("{title}", title)
        .replace("{header}", &nvbar_contents)
        .replace("{content}", &body)
        .replace("{footer}", &footer_content);

    Ok(full)
}

pub fn render_writings(posts: &[PostMeta], templates: &Templates, yaml_config: &str) -> anyhow::Result<String> {
    let items: String = posts
        .iter()
        .map(|p| {
            let date = p
                .date
                .as_deref()
                .map(|d| format!("\n      <span class=\"post-card-date\">{}</span>", d))
                .unwrap_or_default();
            crate::scaffold::CARD_HTML
                .replace("{url}", &p.url)
                .replace("{title}", &p.title)
                .replace("{date}", &date)
        })
        .collect::<Vec<_>>()
        .join("\n    ");

    let footer_links = update_footer(yaml_config);
    let footer_content = templates.footer.replace("{links}", &footer_links);

    let nav_bar = get_site_path(yaml_config);
    let nvbar_contents = templates.header.replace("{links}", &nav_bar);

    let body = templates.writings.replace("{posts}", &items);

    let full = templates
        .base
        .replace("{title}", "Writings")
        .replace("{header}", &nvbar_contents)
        .replace("{content}", &body)
        .replace("{footer}", &footer_content);

    Ok(full)
}

pub fn render_post(input: &str, templates: &Templates, yaml_config: &str) -> anyhow::Result<String> {
    let (fm, content) = crate::utils::parse_content(input)?;

    let html_content = md_to_html(&content);

    let title = fm.as_ref().map(|f| f.title.as_str()).unwrap_or("Untitled");
    let footer_links = update_footer(yaml_config);
    let footer_content = templates.footer.replace("{links}", &footer_links);

    let nav_bar = get_site_path(yaml_config);
    let nvbar_contents = templates.header.replace("{links}", &nav_bar);

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
        .replace("{header}", &nvbar_contents)
        .replace("{content}", &body)
        .replace("{footer}", &footer_content);

    Ok(full)
}
