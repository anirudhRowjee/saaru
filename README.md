# Saaru

Saaru is now an SSG Written in Rust. It uses Jinja Templates to render, along with Markdown to maintain content.

```
   ____
  / __/__ ____ _______ __
 _\ \/ _ `/ _ `/ __/ // /
/___/\_,_/\_,_/_/  \_,_/

A Static Site Generator for Fun and Profit
```

### Etymology

Saaru means Rasam, which is a type of spicy, thin lentil soup, often eaten with rice. This project is called Saaru because I like Saaru very much.

### Nonsense Formal explanation

SAARU -> StAtic Almanac Renderer and Unifier

## Feature map

- [x] Render a single markdown file to HTML
- [x] Render a single markdown file to HTML with a Jinja Template
- [x] URGENT: Remove frontmatter from being rendered in the HTML
- [x] Make templates readable from a single directory
  - [x] URGENT: This is probably a good time to add - [x] Reference folder structure - [x] Command Line Arguments (to pass in path to reference folder structure)
  - [x] Make template readable from frontmatter
  - [x] Solve for nested templates
- [x] Render a directory structure of Markdown and Jinja to a directory structure of HTML
- [ ] Run Pre-flight checks (check if templates dir exists, check if source dir exists, etc)
- [ ] External CSS / Custom CSS injection
- [ ] Parallelized rendering (see HACK comments)

- [x] Think about [Deep Data Merge](https://www.11ty.dev/docs/data/)

  - [x] Think about single-tree-pass DDM Data Sourcing (implemented in the 2-pass method)
  - [x] Collect frontmatter data
  - [x] Implement Deep Data Merge for Tags
  - [x] Implement Deep Data Merge for Collections

- [ ] Web server + Live reload?
- [ ] tree-shaken rendering, only re-render what's changed?
  - [ ] Merkle Tree based hash checks?

### Rearchitecting - wipe

~~How will I make the codebase work with all these features?~~

I DID IT!

## Data Merge Architecture

Each and every `.md` file has frontmatter, which it uses to determine which collections it's a part of.

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

### The Frontmatter Map

This is the primary index for every file being considered in the static site generator.

### New Render Pipeline

This might need to be rearchitected to include live reload when it happens.

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
   - [ ] Now the entire set of templates has access to the data acquired in the merge.
