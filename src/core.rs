use anyhow::{Result, anyhow};
use chrono::Local;
use notify::{RecursiveMode, Watcher};
use serde_json;
use std::{
    fs,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tiny_http::{Header, Response, Server};

use crate::{
    scaffold::{
        BASE_HTML, DEFAULT_KURO_YAML, FOOTER_HTML, HEADER_HTML, INDEX_CSS, INDEX_JS, INDEX_MD,
        PAGE_HTML, POST_HTML, RESET_CSS, WRITINGS_HTML, first_post_md,
    },
    types::{PostMeta, Project, Templates},
    utils::{build_header_html, copy_dir, render_page, render_post, render_writings},
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

    let today = Local::now().format("%Y-%m-%d").to_string();
    fs::write(project.source_dir.join("index.md"), INDEX_MD)?;
    fs::write(
        project.source_dir.join("writings/first-post.md"),
        first_post_md(&today),
    )?;

    fs::write(project.assets_dir.join("index.css"), INDEX_CSS)?;
    fs::write(project.assets_dir.join("reset.css"), RESET_CSS)?;
    fs::write(project.assets_dir.join("index.js"), INDEX_JS)?;
    println!("  ✓ Site theme assets written");

    fs::write(templates_dir.join("base.html"), BASE_HTML)?;
    fs::write(templates_dir.join("header.html"), HEADER_HTML)?;
    fs::write(templates_dir.join("footer.html"), FOOTER_HTML)?;
    fs::write(templates_dir.join("page.html"), PAGE_HTML)?;
    fs::write(templates_dir.join("post.html"), POST_HTML)?;
    fs::write(templates_dir.join("writings.html"), WRITINGS_HTML)?;
    println!("  ✓ Created templates\n");

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
        return Err(anyhow!("file already exists: {}", path.display()));
    }

    let today = Local::now().format("%Y-%m-%d").to_string();
    let content = format!("---\ntitle: \"{}\"\ndate: \"{}\"\n---\n\n", name, today);
    fs::write(&path, content)?;
    println!("\n  ✓ Created {}\n", path.display());

    Ok(())
}

pub fn build_site(project: &Project) -> Result<()> {
    build_site_inner(project, false)
}

fn build_site_inner(project: &Project, silent: bool) -> Result<()> {
    // clean dist if it exists
    if project.out_dir.exists() {
        fs::remove_dir_all(&project.out_dir)?;
    }
    fs::create_dir_all(&project.out_dir)?;

    copy_dir(&project.assets_dir, &project.out_dir)?;
    fs::write(project.out_dir.join("index.css"), INDEX_CSS)?;
    fs::write(project.out_dir.join("reset.css"), RESET_CSS)?;
    fs::write(project.out_dir.join("index.js"), INDEX_JS)?;

    let mut templates = Templates::load(&project.root)?;
    if !silent {
        println!("\n  ✓ Loaded templates");
    }

    // Scan non-index pages in content/ to build the navbar
    let mut page_entries: Vec<_> = fs::read_dir(&project.source_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension().and_then(|s| s.to_str()) == Some("md")
                && p.file_stem().and_then(|s| s.to_str()) != Some("index")
        })
        .collect();
    page_entries.sort_by_key(|e| e.file_name());

    let extra_pages: Vec<(String, String)> = page_entries
        .iter()
        .map(|e| {
            let path = e.path();
            let name = path.file_stem().unwrap().to_str().unwrap().to_string();
            let content = fs::read_to_string(&path).unwrap_or_default();
            let title = crate::utils::parse_content(&content)
                .ok()
                .and_then(|(fm, _)| fm)
                .map(|f| f.title)
                .unwrap_or_else(|| name.clone());
            (title, format!("/{}/", name))
        })
        .collect();

    templates.header = build_header_html(&extra_pages);

    let index_md = project.source_dir.join("index.md");
    let index_html = project.out_dir.join("index.html");
    let yaml_config_path = {
        let p = project.root.join("kuro.yaml");
        if p.exists() { p } else { project.root.join("kuro.yml") }
    };

    let yaml_config = fs::read_to_string(yaml_config_path)?;

    let content = fs::read_to_string(index_md)?;
    let html = render_page(&content, &templates, &yaml_config)?;
    fs::write(index_html, html)?;

    // Build extra pages to dist/{name}/index.html
    for entry in &page_entries {
        let path = entry.path();
        let name = path.file_stem().unwrap().to_str().unwrap();
        let content = fs::read_to_string(&path)?;
        let html = render_page(&content, &templates, &yaml_config)?;
        let page_dir = project.out_dir.join(name);
        fs::create_dir_all(&page_dir)?;
        fs::write(page_dir.join("index.html"), html)?;
    }

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
            let html = render_post(&content, &templates, &yaml_config)?;
            let output_path = writings_dst.join(format!("{}.html", file_name));
            fs::write(output_path, html)?;

            posts.push(PostMeta {
                title: fm
                    .as_ref()
                    .map(|f| f.title.clone())
                    .unwrap_or_else(|| file_name.clone()),
                date: fm.and_then(|f| f.date),
                url: format!("/writings/{}.html", file_name),
            });
        }
    }

    // sort by date descending, undated posts last
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    let json = serde_json::to_string_pretty(&posts)?;
    fs::write(writings_dst.join("index.json"), &json)?;

    let writings_index_html = render_writings(&posts, &templates, &yaml_config)?;
    fs::write(writings_dst.join("index.html"), writings_index_html)?;

    if !silent {
        println!("\n  ✓ Built writings\n");
    }

    Ok(())
}

const RELOAD_SCRIPT: &str = r#"<script>
(function(){
  var v=null;
  setInterval(async function(){
    try{
      var r=await fetch('/_kuro_reload');
      var d=await r.json();
      if(v===null)v=d.version;
      else if(d.version!==v)location.reload();
    }catch(e){}
  },500);
})();
</script>"#;

pub fn serve(project: &Project, watch: bool) -> Result<()> {
    build_site(project)?;
    println!("  ✓ Site built");

    let reload_version = Arc::new(AtomicU64::new(0));

    if watch {
        let version = reload_version.clone();
        let proj = project.clone();
        std::thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();
            let mut watcher =
                notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                    tx.send(res).ok();
                })
                .expect("watcher");
            watcher
                .watch(&proj.source_dir, RecursiveMode::Recursive)
                .ok();
            watcher
                .watch(&proj.assets_dir, RecursiveMode::Recursive)
                .ok();
            watcher
                .watch(&proj.template_dir, RecursiveMode::Recursive)
                .ok();

            loop {
                match rx.recv() {
                    Ok(Ok(event)) if !event.kind.is_access() => {
                        // debounce: drain any queued events
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        while rx.try_recv().is_ok() {}

                        let t = std::time::Instant::now();
                        match build_site_inner(&proj, true) {
                            Ok(_) => {
                                version.fetch_add(1, Ordering::SeqCst);
                                println!("  ✓ Rebuilt in {}ms", t.elapsed().as_millis());
                            }
                            Err(e) => eprintln!("  ✗ Build error: {e}"),
                        }
                    }
                    Err(_) => break,
                    _ => {}
                }
            }
        });
        println!("  ✓ Watching for changes");
    }

    let addr = "127.0.0.1:3000";
    let server = Server::http(addr).map_err(|e| anyhow!(e))?;
    println!("  ✓ Serving at localhost:3000");

    let json_ct: Header = "Content-Type: application/json".parse().unwrap();
    let html_ct: Header = "Content-Type: text/html; charset=utf-8".parse().unwrap();

    for request in server.incoming_requests() {
        let url = request.url().to_string();

        if watch && url == "/_kuro_reload" {
            let v = reload_version.load(Ordering::SeqCst);
            let body = format!("{{\"version\":{v}}}");
            request.respond(Response::from_string(body).with_header(json_ct.clone()))?;
            continue;
        }

        let path = if url == "/" {
            project.out_dir.join("index.html")
        } else {
            let trimmed = url.trim_start_matches('/');
            project.out_dir.join(trimmed)
        };

        let path = if path.is_dir() {
            path.join("index.html")
        } else {
            path
        };

        let is_html = path.extension().and_then(|e| e.to_str()) == Some("html");

        match fs::read(&path) {
            Ok(data) if watch && is_html => {
                let mut html = String::from_utf8_lossy(&data).into_owned();
                html = html.replace("</body>", &format!("{RELOAD_SCRIPT}</body>"));
                request.respond(Response::from_string(html).with_header(html_ct.clone()))?;
            }
            Ok(data) => {
                request.respond(Response::from_data(data))?;
            }
            Err(_) => {
                request.respond(Response::from_string("404 Not Found").with_status_code(404))?;
            }
        }
    }

    Ok(())
}
