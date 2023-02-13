---
title: Getting Started with Saaru
description: Let's build our first site with Saaru!
wip: false
template: post.jinja
tags:
  - documentation
  - saaru
collections:
  - get_started
---

## Prerequisites

1. Have Rust installed!
2. Clone this repository

## Let's Build our first site with Saaru!

This tutorial aims to be as self-contained as possible, so you'll find file dumps that you can copy-paste into specific locations. Ideally, all you'll need is the `saaru` binary to follow along, and nothing else!

### Folder Structure

Saaru is slightly opinionated about how you structure your source.

Use these commands to generate the necessary folder structure - this is a test of the live reload.

this is nice!

```bash
$ mkdir -p my_new_website/{src,static,templates}
$ touch my_new_website/{src/hello_world.md,templates/{base,tags,tags_page}.jinja,.saaru.json}
```

```lisp
my_new_website
  |- .saaru.json
  |- src/
  |- |- hello_world.md
  |- templates/
  |- |- base.jinja
  |- |- tags.jinja
  |- |- tags_page.jinja
  |- static/
```

### Files

We'll walk through each file and its use in the static site generator.

- `.saaru.json`

  This is the metadata store for the entire static site generator. Go ahead and copy-paste the following into the file -

  ```json
  {
    "metadata": {
      "author": {
        "name": "Author",
        "one_line_desc": "hello, world!",
        "twitter": "twitter.com/username",
        "github": "github.com/username"
      },
      "templates": {
        "default": "base.jinja"
      }
    }
  }
  ```

- `src/hello_world.md`

  Here's the first piece of content on the site. The structure of the `src` folder is arbitrary, and you can go as deep as you want with your folder/file structure. Each of these paths will be rendered 1:1 into your website, so you don't have to worry about routing.

  Copy-paste the following into the `src/hello_world.md` -

  ```md
  ---
  title: Hello, world!
  description: Welcome to Saaru!
  wip: false
  tags:
    - base
    - intro
    - saaru
  collections:
    - index_pages
  ---

  Hello, world!
  ```

- `templates/base.jinja`

  We can use every feature of Jinja Templates (such as interpolation, filters, etc) to make our site better. Note that we're referencing things from the post in `{{postcontent | safe}}` to get our markdown content into the template. Copy-paste the following -

  ```html
  <!DOCTYPE html>
  <html>
    <head>
      <meta charset="utf-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1" />
    </head>
    <body>
      <main class="container">
        <h5>Website - {{ base.json.metadata.author.name }}</h5>
        {{ postcontent | safe }}
      </main>
    </body>
  </html>
  ```

- `templates/tags.jinja`
  This marks our first attempt using the deep data merge - tags - to access the tags across the entire site, as well as the list of posts using these tags. This page is also the list of all the tags we have in the website, and can be found at `/tags`

  ```html
  <h1>Tags</h1>
  <div>
    {% for tag in base.tags %}
    <a class="tag" href="/tags/{{tag}}"> {{tag}}</a>
    {% endfor %}
  </div>
  ```

- `templates/tags_page.jinja`
  This template is necessary as Saaru automatically generates pages for each and every tag. Here, `posts` refers to the list of posts for each tag. This is a compulsory variable, and should not be changed.

  ```html
  <h1>Pages Tagged {{tag}}</h1>
  <div>
    <ul>
      {% for post in base.posts %}
      <li><a href="{{post.link}}"> {{post.frontmatter.title}}</a></li>
      {% endfor %}
    </ul>
  </div>
  ```

### Next Steps

Once you're done doing this, head over to the [usage](/usage) section of the site to render your website - but if you don't want to, just do the following -

1. Navigate to the cloned repository
2. Run the following -

```bash
$ cargo run --release -- --base-path <path to my_new_website>
```

3. Your website should now be in `<path to my_new_website>/build/`.

```
$ tree build

build/
├── hello_world.html
└── tags
    ├── base.html
    ├── index.html
    ├── intro.html
    └── saaru.html

1 directory, 5 files
```

4. You can use an application like [serve](https://www.npmjs.com/package/serve) to then start a web server, which will make your website accessible via the browser.

```bash
$ cd build/
$ serve
```

5. Rejoice!

Hello, Adarsh! This is a hello form the live reload. Isn't this nice?
