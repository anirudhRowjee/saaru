---
title: Frontmatter
description: Understand how the frontmatter of your markdown can be structured
wip: false
template: post.jinja
tags:
  - documentation
  - posts
  - saaru
collections:
  - internals
---

### Internal Definition

```rust
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub description: Option<String>,
    pub date: Option<String>,
    pub tags: Option<Vec<String>>,
    pub collections: Option<Vec<String>>,
    pub wip: Option<bool>,
    pub template: Option<String>,
    pub link: Option<String>,
    pub meta: Option<Value>,
}
```

All these fields are optional.

- `meta` is any arbitrary JSON you wish to tack on to each post.
