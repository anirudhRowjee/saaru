---
title: Deep Data Merge
description: Understand how the deep data merge works
wip: false
template: post.jinja
tags:
  - documentation
  - posts
  - saaru
collections:
  - internals
---

The Deep Data Merge is what allows Saaru sites to access various metadata from other posts, including ways of organizing your posts.

Saaru also allows you to define your own systems of classification, such as [tags](/internals/tags) or [collections](/internals/collections). Both are functionally similar, but you can choose to opt out of either or both by simply not including the `tags` or `collections` fields in your frontmatter per your choice.

Furthermore, Saaru also allows you to have arbitrary JSON Values injected in through the `.saaru.json` file, which can then be accessed by all templates - one use-case might be that you pull in some data from an API as a part of your build stage, store it in `.saaru.json`, and then use that content in the site.

All this information can be accessed during the rendering of any template. This is known as the Deep Data Merge.

## Motivations

Each and every `.md` file has frontmatter, which it uses to store metadata such as the author name, date, title, description, and so on.

Saaru also allows you to define your own systems of classification, such as [tags](/internals/tags) or [collections](/internals/collections). Both are functionally similar, but you can choose to opt out of either or both by simply not including the `tags` or `collections` fields in your frontmatter per your choice.

```yaml
---
title: something
description: something else
collections:
  - posts
tags:
  - computerscience
  - space
  - alphabet
---
```

The first pass of the SSG Renderer is a frontmatter pass, where inverted indices (think TF-IDF) of both collections and tags are created. These indices are then accessible in frontmatter such that one can easily generate an index page for every document present here.

Thus, our generated indices will look something like this ->

```yaml
collection_map:
  - posts: [something]
tag_map:
  - computerscience: [something]
  - space: [something]
  - alphabet: [something]
```

It is also possible to auto-generate the tags pages such that lookups are possible on the basis of tags.

## New Render Pipeline

In this architecture, there are two passes that go into rendering files - this is architectured to make the deep data merge possible.

1. Frontmatter Pass -
   - Read every File in the source directory
   - Capture all the frontmatter in structs
   - Read all the Markdown Content and store it (but do not convert it to HTML) - this is so we don't need to read it again to save on I/O
   - Create all the indices/maps on the fly (collection_map, tag_map) at this point
   - Capture structure - `HashMap<Path, AugmentedFrontmatter>` where `AugmentedFrontmatter` has the frontmatter, read content markdown (and possibly later have live reload based on file changes)

> By the end of the frontmatter pass, all collections and collection data must be satisfied, should any other template wish to read it

2. Render Pass
   - Iterate through the entire hashmap, passing the collections available as a part of the context
   - Render the markdown present and write it to file
   - Now the entire set of templates has access to the data acquired in the merge.

## The Frontmatter passed to every file

Every file gets the following frontmatter passed into it ->

```rust
let rendered_final_html = rendered_template
.render(context!(
      frontmatter => input_aug_frontmatter.frontmatter,
      postcontent => html_output,
      tags => &self.tag_map,
      collections => &self.collection_map,
      json => &self.arguments.json_content
      ))
.unwrap();
```

- `frontmatter` -> The frontmatter for the post
- `postcontent` -> The parsed markdown and HTML for the post
- `tags` -> The tags sitewide, structured as `Vec<tag: String, Vec<Post: String>>`
- `collections` -> The collections sitewide, structured as `Vec<collection: String, Vec<Post: String>>`
- `json` -> Arbitrary JSON passed in through `.saaru.json`, accessible sitewide

Feel free to use any of these in your pages!
