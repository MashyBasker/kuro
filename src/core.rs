use anyhow::{Result, anyhow};
use serde_json;
use std::{fs, path::Path};
use tiny_http::{Response, Server};

use crate::{
    scaffold::{
        BASE_HTML, DEFAULT_KURO_YAML, FIRST_POST_MD, FOOTER_HTML, HEADER_HTML, INDEX_CSS, INDEX_JS,
        INDEX_MD, PAGE_HTML, POST_HTML, RESET_CSS, WRITINGS_HTML,
    },
    types::{PostMeta, Project, Templates},
    utils::{copy_dir, render_page, render_post, render_writings},
};

pub fn create_site_directory(root_path: &Path) -> Result<Project> {
    fs::create_dir_all(root_path)?;
    let project = Project::new(root_path.to_path_buf());
    println!("\n  ✓ Initialized new Kuro site");

    fs::create_dir_all(&project.source_dir)?;
    println!("  ✓ Site source created");

    let writings_dir = project.source_dir.join("writings");
    fs::create_dir_all(&writings_dir)?;
    println!("  ✓ Writings directory generated");

    fs::create_dir_all(&project.assets_dir)?;
    println!("  ✓ Site theme assets generated");

    fs::create_dir_all(&project.out_dir)?;
    println!("  ✓ Site output directory created");

    let templates_dir = &project.template_dir;

    fs::create_dir_all(templates_dir)?;
    println!("  ✓ Templated directory created");

    fs::write(project.root.join("kuro.yml"), DEFAULT_KURO_YAML)?;
    println!("  ✓ Kuro config file generated");

    fs::write(project.source_dir.join("index.md"), INDEX_MD)?;
    fs::write(
        project.source_dir.join("writings/first-post.md"),
        FIRST_POST_MD,
    )?;

    fs::write(project.assets_dir.join("index.css"), INDEX_CSS)?;
    fs::write(project.assets_dir.join("reset.css"), RESET_CSS)?;
    fs::write(project.assets_dir.join("index.js"), INDEX_JS)?;
    println!("  ✓ Site theme assets written\n");

    fs::write(templates_dir.join("base.html"), BASE_HTML)?;
    fs::write(templates_dir.join("header.html"), HEADER_HTML)?;
    fs::write(templates_dir.join("footer.html"), FOOTER_HTML)?;
    fs::write(templates_dir.join("page.html"), PAGE_HTML)?;
    fs::write(templates_dir.join("post.html"), POST_HTML)?;
    fs::write(templates_dir.join("writings.html"), WRITINGS_HTML)?;
    println!("✓ Created templates");

    Ok(project)
}

pub fn create_new_file(project: &Project, name: &str, post: bool) -> Result<()> {
    let dir = if post {
        project.source_dir.join("writings")
    } else {
        project.source_dir.clone()
    };

    let path = dir.join(format!("{}.md", name));

    if path.exists() {
        println!("\n  ✗ File already exists: {}\n", path.display());
        return Ok(());
    }

    let content = format!("---\ntitle: \"{}\"\ndate:\n---\n\n", name);
    fs::write(&path, content)?;
    println!("\n  ✓ Created {}\n", path.display());

    Ok(())
}

pub fn build_site(project: &Project) -> Result<()> {
    // clean dist if it exists
    if project.out_dir.exists() {
        fs::remove_dir_all(&project.out_dir)?;
    }
    fs::create_dir_all(&project.out_dir)?;

    copy_dir(&project.assets_dir, &project.out_dir)?;

    let templates = Templates::load(&project.root)?;
    println!("  ✓ Loaded templates");

    let index_md = project.source_dir.join("index.md");
    let index_html = project.out_dir.join("index.html");

    let content = fs::read_to_string(index_md)?;
    let html = render_page(&content, &templates)?;

    fs::write(index_html, html)?;

    // 4. Build writings/
    let writings_src = project.source_dir.join("writings");
    let writings_dst = project.out_dir.join("writings");

    fs::create_dir_all(&writings_dst)?;

    let mut posts: Vec<PostMeta> = Vec::new();

    for entry in fs::read_dir(writings_src)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&path)?;
            let (fm, _) = crate::utils::parse_content(&content)?;

            let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
            let html = render_post(&content, &templates)?;
            let output_path = writings_dst.join(format!("{}.html", file_name));
            fs::write(output_path, html)?;

            posts.push(PostMeta {
                title: fm.as_ref().map(|f| f.title.clone()).unwrap_or_else(|| file_name.clone()),
                date: fm.and_then(|f| f.date),
                url: format!("/writings/{}.html", file_name),
            });
        }
    }

    // sort by date descending, undated posts last
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    let json = serde_json::to_string_pretty(&posts)?;
    fs::write(writings_dst.join("index.json"), &json)?;

    let writings_index_html = render_writings(&posts, &templates)?;
    fs::write(writings_dst.join("index.html"), writings_index_html)?;

    println!("\n  ✓ Built writings\n");

    Ok(())
}

pub fn serve(project: &Project) -> Result<()> {
    // 1. Build first
    build_site(project)?;
    println!("  ✓ Site built");

    let addr = "127.0.0.1:3000";
    let server = Server::http(addr).map_err(|e| anyhow!(e))?;

    println!("  ✓ Serving at localhost:3000");

    for request in server.incoming_requests() {
        let url = request.url();

        let path = if url == "/" {
            project.out_dir.join("index.html")
        } else {
            // strip leading '/'
            let trimmed = url.trim_start_matches('/');
            project.out_dir.join(trimmed)
        };

        let path = if path.is_dir() {
            path.join("index.html")
        } else {
            path
        };

        let response = match fs::read(&path) {
            Ok(data) => Response::from_data(data),
            Err(_) => Response::from_string("404 Not Found").with_status_code(404),
        };

        request.respond(response)?;
    }

    Ok(())
}
