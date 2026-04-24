use anyhow::{Result, anyhow};
use chrono::Local;
use notify::{RecursiveMode, Watcher};
use serde_json;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tiny_http::{Header, Response, Server};

use crate::{
    render::SiteRenderer,
    types::{PostMeta, Templates},
    utils::{build_header_html, copy_dir, parse_content},
};

pub const DEFAULT_KURO_YAML: &str = include_str!("../assets/templates/kuro.yml");
pub const INDEX_MD: &str = include_str!("../assets/templates/index.md");
pub const BASE_HTML: &str = include_str!("../assets/templates/base.html");
pub const HEADER_HTML: &str = include_str!("../assets/templates/header.html");
pub const FOOTER_HTML: &str = include_str!("../assets/templates/footer.html");
pub const PAGE_HTML: &str = include_str!("../assets/templates/page.html");
pub const POST_HTML: &str = include_str!("../assets/templates/post.html");
pub const CARD_HTML: &str = include_str!("../assets/templates/card.html");
pub const WRITINGS_HTML: &str = include_str!("../assets/templates/writings.html");

// These theme files are taken from owickstrom/the-monospace-web
pub const INDEX_CSS: &str = include_str!("../assets/themes/index.css");
pub const INDEX_JS: &str = include_str!("../assets/themes/index.js");
pub const RESET_CSS: &str = include_str!("../assets/themes/reset.css");

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

#[derive(Debug, PartialEq, Clone)]
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

    pub fn init(root_path: &Path) -> Result<Self> {
        fs::create_dir_all(root_path)?;
        let project = Self::new(root_path.to_path_buf());
        println!("\n  ✓ Initialized new Kuro site");

        fs::create_dir_all(&project.source_dir)?;
        println!("  ✓ Site source created");

        fs::create_dir_all(project.source_dir.join("writings"))?;
        println!("  ✓ Writings directory generated");

        fs::create_dir_all(&project.assets_dir)?;
        println!("  ✓ Site theme assets generated");

        fs::create_dir_all(&project.out_dir)?;
        println!("  ✓ Site output directory created");

        fs::create_dir_all(&project.template_dir)?;
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

        fs::write(project.template_dir.join("base.html"), BASE_HTML)?;
        fs::write(project.template_dir.join("header.html"), HEADER_HTML)?;
        fs::write(project.template_dir.join("footer.html"), FOOTER_HTML)?;
        fs::write(project.template_dir.join("page.html"), PAGE_HTML)?;
        fs::write(project.template_dir.join("post.html"), POST_HTML)?;
        fs::write(project.template_dir.join("writings.html"), WRITINGS_HTML)?;
        println!("  ✓ Created templates\n");

        Ok(project)
    }

    pub fn new_file(&self, name: &str, post: bool) -> Result<()> {
        let dir = if post {
            self.source_dir.join("writings")
        } else {
            self.source_dir.clone()
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

    pub fn build(&self) -> Result<()> {
        if self.out_dir.exists() {
            fs::remove_dir_all(&self.out_dir)?;
        }
        fs::create_dir_all(&self.out_dir)?;

        copy_dir(&self.assets_dir, &self.out_dir)?;

        fs::write(self.out_dir.join("index.css"), INDEX_CSS)?;
        fs::write(self.out_dir.join("reset.css"), RESET_CSS)?;
        fs::write(self.out_dir.join("index.js"), INDEX_JS)?;

        let mut templates = Templates::load(&self.root)?;

        println!("\n  ✓ Loaded templates");

        // Scan non-index pages in content/ to build the navbar
        let mut page_entries: Vec<_> = fs::read_dir(&self.source_dir)?
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
                let title = parse_content(&content)
                    .ok()
                    .and_then(|(fm, _)| fm)
                    .map(|f| f.title)
                    .unwrap_or_else(|| name.clone());
                (title, format!("/{}/", name))
            })
            .collect();

        templates.header = build_header_html(&extra_pages);

        let index_md = self.source_dir.join("index.md");
        let index_html = self.out_dir.join("index.html");
        let yaml_config_path = {
            let p = self.root.join("kuro.yaml");
            if p.exists() {
                p
            } else {
                self.root.join("kuro.yml")
            }
        };

        let yaml_config = fs::read_to_string(yaml_config_path)?;
        let renderer = SiteRenderer::new(templates, &yaml_config)?;

        let content = fs::read_to_string(index_md)?;
        fs::write(index_html, renderer.render_page(&content)?)?;

        // Build extra pages to dist/{name}/index.html
        for entry in &page_entries {
            let path = entry.path();
            let name = path.file_stem().unwrap().to_str().unwrap();
            let content = fs::read_to_string(&path)?;
            let page_dir = self.out_dir.join(name);
            fs::create_dir_all(&page_dir)?;
            fs::write(page_dir.join("index.html"), renderer.render_page(&content)?)?;
        }

        // Build writings/
        let writings_src = self.source_dir.join("writings");
        let writings_dst = self.out_dir.join("writings");

        fs::create_dir_all(&writings_dst)?;

        let mut posts: Vec<PostMeta> = Vec::new();

        for entry in fs::read_dir(writings_src)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let content = fs::read_to_string(&path)?;
                let (fm, _) = parse_content(&content)?;

                let file_name = path.file_stem().unwrap().to_str().unwrap().to_string();
                let output_path = writings_dst.join(format!("{}.html", file_name));
                fs::write(output_path, renderer.render_post(&content)?)?;

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

        posts.sort_by(|a, b| b.date.cmp(&a.date));

        let json = serde_json::to_string_pretty(&posts)?;
        fs::write(writings_dst.join("index.json"), &json)?;

        fs::write(writings_dst.join("index.html"), renderer.render_writings(&posts)?)?;

        println!("\n  ✓ Built writings\n");

        Ok(())
    }

    pub fn serve(self, watch: bool) -> Result<()> {
        self.build()?;
        println!("  ✓ Site built");

        let reload_version = Arc::new(AtomicU64::new(0));

        if watch {
            let version = reload_version.clone();
            let proj = self.clone();
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
                            match proj.build() {
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
        let css_ct: Header = "Content-Type: text/css; charset=utf-8".parse().unwrap();
        let js_ct: Header = "Content-Type: text/javascript; charset=utf-8".parse().unwrap();

        for request in server.incoming_requests() {
            let url = request.url().to_string();

            if watch && url == "/_kuro_reload" {
                let v = reload_version.load(Ordering::SeqCst);
                let body = format!("{{\"version\":{v}}}");
                request.respond(Response::from_string(body).with_header(json_ct.clone()))?;
                continue;
            }

            let path = if url == "/" {
                self.out_dir.join("index.html")
            } else {
                let trimmed = url.trim_start_matches('/');
                self.out_dir.join(trimmed)
            };

            let path = if path.is_dir() {
                path.join("index.html")
            } else {
                path
            };

            let ext = path.extension().and_then(|e| e.to_str());

            match fs::read(&path) {
                Ok(data) if watch && ext == Some("html") => {
                    let mut html = String::from_utf8_lossy(&data).into_owned();
                    html = html.replace("</body>", &format!("{RELOAD_SCRIPT}</body>"));
                    request.respond(Response::from_string(html).with_header(html_ct.clone()))?;
                }
                Ok(data) => {
                    let ct = match ext {
                        Some("html") => Some(html_ct.clone()),
                        Some("css") => Some(css_ct.clone()),
                        Some("js") => Some(js_ct.clone()),
                        Some("json") => Some(json_ct.clone()),
                        _ => None,
                    };
                    let response = Response::from_data(data);
                    match ct {
                        Some(header) => request.respond(response.with_header(header))?,
                        None => request.respond(response)?,
                    }
                }
                Err(_) => {
                    request
                        .respond(Response::from_string("404 Not Found").with_status_code(404))?;
                }
            }
        }

        Ok(())
    }
}

fn first_post_md(date: &str) -> String {
    format!(
        "---\ntitle: \"First Post\"\ndate: \"{}\"\n---\n\nHello friend.\n\n## Section\n\n- item one\n- item two\n",
        date
    )
}
