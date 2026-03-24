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

// These theme files are taken from owickstrom/the-monospace-web
pub const INDEX_CSS: &str = include_str!("../assets/index.css");
pub const INDEX_JS: &str = include_str!("../assets/index.js");
pub const RESET_CSS: &str = include_str!("../assets/reset.css");
