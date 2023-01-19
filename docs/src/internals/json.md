---
title: JSON
description: Understand how the JSON Content injection works
wip: false
template: post.jinja
tags:
  - documentation
  - posts
  - saaru
collections:
  - internals
---

This is the default JSON used if there's no `.saaru.json` present in the base folder. The field `metadata.templates.default` field is compulsory, or else Saaru will look for a `post.jinja` in your template environment.

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
      "default": "post.jinja"
    }
  }
}
```
