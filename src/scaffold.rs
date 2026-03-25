pub const DEFAULT_KURO_YAML: &str = r#"
name: "My site"
author: "Your name"
description: "New Kuro Website"
url: "http://localhost:3000"
"#;

pub const INDEX_MD: &str = r#"---
title: "Index"
date:
---

Welcome to my site.

This is my personal website powered by **kuro**.

## Posts

- [First Post](/writings/first-post.html)
"#;

pub const FIRST_POST_MD: &str = r#"---
title: "First Post"
date:
---

Hello friend.

## Section

- item one
- item two
"#;

pub const BASE_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>{title}</title>
  <link rel="stylesheet" href="/reset.css">
  <link rel="stylesheet" href="/index.css">
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css">
  <link rel="stylesheet" media="(prefers-color-scheme: light)" href="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.9.0/build/styles/base16/grayscale-light.min.css">
  <link rel="stylesheet" media="(prefers-color-scheme: dark)" href="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.9.0/build/styles/base16/grayscale-dark.min.css">
  <style>
    body { display: flex; flex-direction: column; min-height: 100vh; }
    main { flex: 1; }
    header { display: flex; justify-content: flex-end; align-items: center; margin-bottom: calc(var(--line-height) * 2); }
    nav { display: flex; gap: 2ch; margin: 0; }
    nav a { margin: 0; text-decoration: none; }
    nav a.active { text-decoration: underline; text-decoration-thickness: var(--border-thickness); }
  </style>
</head>
<body>

{header}

<main>
{content}
</main>

{footer}

<script src="/index.js"></script>
<script src="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.9.0/build/highlight.min.js"></script>
<script>hljs.highlightAll();</script>
<script>
  document.querySelectorAll('nav a').forEach(function(link) {
    var href = link.getAttribute('href');
    var path = window.location.pathname;
    if (href === '/' ? path === '/' : path.startsWith(href)) {
      link.classList.add('active');
    }
  });
</script>
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js"></script>
<script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/contrib/auto-render.min.js"
  onload="renderMathInElement(document.body, {
    delimiters: [
      {left: '$$', right: '$$', display: true},
      {left: '$', right: '$', display: false}
    ]
  })">
</script>
</body>
</html>
"#;

pub const HEADER_HTML: &str = r#"
<header>
  <nav>
    <a href="/">Home</a>
    <a href="/writings/">Writings</a>
  </nav>
</header>
"#;

pub const FOOTER_HTML: &str = r#"
<footer>
  <p>© 2026 My Site</p>
</footer>
"#;

pub const PAGE_HTML: &str = r#"
<article>
{content}
</article>
"#;

pub const POST_HTML: &str = r#"
<article>
  <h1>{title}</h1>
  {date}
  {content}
</article>
"#;

pub const WRITINGS_HTML: &str = r#"
<article>
  <h1>Writings</h1>
  <ul>
    {posts}
  </ul>
</article>
"#;

// These theme files are taken from owickstrom/the-monospace-web
pub const INDEX_CSS: &str = include_str!("../assets/index.css");
pub const INDEX_JS: &str = include_str!("../assets/index.js");
pub const RESET_CSS: &str = include_str!("../assets/reset.css");
