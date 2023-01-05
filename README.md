# Saaru

Saaru is now an SSG Written in Rust. It uses Jinja Templates to render, along with Markdown to maintain content.

```
   ____
  / __/__ ____ _______ __
 _\ \/ _ `/ _ `/ __/ // /
/___/\_,_/\_,_/_/  \_,_/

A Static Site Generator for Fun and Profit
```

## Usage

Running the program is simple. Once you have the repository cloned, you can use the following command to check oout the example site ->

```bash
$ cargo run -- --base_path <your example_source directory>
```

Feel free to base your site off of the `example_source` directory, which already has a bunch of templates pre-defined for you. It's got my name in there, but TODO Refactor soon enough.

```bash
$ cargo run -- --base_path ./example_source
```

If nothing's wrong, your entire site as HTML and CSS will present itself in the `./example_source/build` directory. From then onwards, all you need to do is launch a web server with `./example_source/build` as the source such as [this package](https://www.npmjs.com/package/serve).

As of right now, Saaru is a little opinioniated on how exactly you should structure your site. As of right now, it boils down to having a folder with the following structure =>

```
example_source/
├── src
│   ├── index2.md
│   ├── index.md
│   ├── new_index.md
│   └── posts
│       ├── index.md
│       └── post1.md
└── templates
    ├── base.jinja
    ├── post_index.jinja
    ├── post.jinja
    ├── post_new.jinja
    ├── tags.jinja
    └── tags_page.jinja
```

It's possible to have an abitrary configuration of files in the `src` folder, so long as you've got each and every markdown document with the right frontmatter.

Here's the absolute minimum frontmatter. (This will be iterated on, but as of now - ) There must be **A MINIMUM OF ONE TAG PER POST**.

```yaml
---
title: <A title for your post>
description: <a description>
template: post.jinja # This must be a valid template from the `templates` directory
tags:
  - <example tag 1>
  - <example tag 2>
collections:
  - <example collection>
  - <example collection 2>
---
```

### Etymology

Saaru means Rasam, which is a type of spicy, thin lentil soup, often eaten with rice. This project is called Saaru because I like Saaru very much.

### Nonsense Formal explanation

SAARU -> StAtic Almanac Renderer and Unifier

## TODO

- [ ] Make all frontmatter optional
- [ ] Static Directory Support (Minify CSS and Build, copy over all other static files)
- [ ] Custom Info JSON File - for defaults, fixed params, etc (Perhaps a `.saaru.json`)

  ```json
  {
    "default_dir": "abc",
    "default_template": "some_template.jinja",
    "author": {
      "name": "Somesh",
      "bio": "this is my bio",
      "twitter": "..."
    }
  }
  ```

- [ ] Run Pre-flight checks (check if templates dir exists, check if source dir exists, etc)
- [ ] External CSS / Custom CSS injection
- [ ] Parallelized rendering
- [ ] Web server + Live reload?
- [ ] tree-shaken rendering, only re-render what's changed?
- [ ] Merkle Tree based hash checks?

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
   - Now the entire set of templates has access to the data acquired in the merge.
