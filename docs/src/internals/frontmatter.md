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

### Render Phase

Every file gets the following frontmatter passed into it ->

```rust
let rendered_final_html = rendered_template
.render(context!(
      frontmatter => input_aug_frontmatter.frontmatter,
      postcontent => html_output,
      tags => &self.tag_map,
      collections => &self.collection_map,
      ))
.unwrap();
```

- `frontmatter` -> The frontmatter for the post
- `postcontent` -> The parsed markdown and HTML for the post
- `tags` -> The tags sitewide, structured as `Vec<tag: String, Vec<Post: String>>`
- `collections` -> The collections sitewide, structured as `Vec<collection: String, Vec<Post: String>>`

Feel free to use any of these in your pages!
