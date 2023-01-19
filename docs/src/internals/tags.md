---
title: Tags
description: Understand how the tag system works
wip: false
template: post.jinja
tags:
  - documentation
  - posts
  - saaru
collections:
  - internals
---

Tags are one of the two ways one can organize their posts in Saaru. They're mentioned in the frontmatter of each post as follows, and are allowed to be a **one-dimensional YAML Array of strings**.

> **NOTE**
> It isn't necessary to have tags in your post, neither is it necessary to have the `tags` field in your frontmatter.

```yaml
title: Tags
description: Understand how the tag system works
wip: false
template: post.jinja
tags: # <-- here!
  - documentation
  - posts
  - saaru
collections:
  - internals
```

Just mentioning the collection that each post belongs to allows you to access the collections and their posts in your templates as follows ->

```jinja
{% for tag in tags %}
  <a class="tag" href="/tags/{{tag}}"> {{tag}}</a>
{% endfor %}
```

> **IMPORTANT**
> Saaru Automatically generates collection pages for tags, accessible by `/tags/<tag name>`
