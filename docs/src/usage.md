---
title: Usage
description: Let's get started using Saaru!
wip: false
template: post.jinja
tags:
  - base
  - intro
  - saaru
collections:
  - get_started
---

### Building a website

Running the program is simple. Once you have the repository cloned, you can use the following command to check oout the example site ->

```bash
$ cargo run --release -- --base_path <your example_source directory>
```

Feel free to base your site off of the `docs` directory, which already has a bunch of templates pre-defined for you. It's got my name in there, but TODO Refactor soon enough.

```bash
$ cargo run --release -- --base_path ./example_source
```

If nothing's wrong, your entire site as HTML and CSS will present itself in the `./docs/build` directory. From then onwards, all you need to do is launch a web server with `./docs/build` as the source such as [this package](https://www.npmjs.com/package/serve).

It's possible to have an abitrary configuration of files in the `src` folder, so long as you've got each and every markdown document with the right frontmatter.

### Live Reload

As of right now, Live reload is enabled by default, and is hidden behind a command line flag.

```bash
$ cargo run --release -- --base_path ./example_source --live-reload
```

As and when you make a change to a file and save the file, Saaru will re-render that file into the build directory. On your browser (or if your web server supports watching the file system, do nothing - ), hit refresh to see your content updated.
