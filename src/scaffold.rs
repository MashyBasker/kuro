pub const DEFAULT_KURO_YAML: &str = r#"
name: "My site"
author: "Your name"
description: "New Kuro Website"
url: "http://localhost:3000"
"#;

pub const INDEX_MD: &str = r#"---
title = "Index"
date = ~
---

Welcome to my site.

This is my personal website powered by **kuro**.

## Posts

- [First Post](/writings/first-post.html)
"#;

pub const FIRST_POST_MD: &str = r#"---
title = "First Post"
date = ~
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
</head>
<body>

{header}

<main>
{content}
</main>

{footer}

<script src="/index.js"></script>
</body>
</html>
"#;

pub const HEADER_HTML: &str = r#"
<header>
  <h1>{title}</h1>
  {date}
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

// These theme files are taken from owickstrom/the-monospace-web
pub const INDEX_CSS: &str = include_str!("../assets/index.css");
pub const INDEX_JS: &str = include_str!("../assets/index.js");
pub const RESET_CSS: &str = include_str!("../assets/reset.css");
