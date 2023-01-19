---
title: Collections
description: Understand how the collections system works
wip: false
template: post.jinja
tags:
  - documentation
  - posts
  - saaru
collections:
  - internals
---

Collections are one of the two ways one can organize their posts in Saaru. They're mentioned in the frontmatter of each `.md` file as follows, and are only allowed to be a **one-dimensional YAML Array of strings**.

> **NOTE**
> It isn't necessary to have your post belong to a collection, neither is it necessary to have the `collections` field in your frontmatter.

```yaml
title: Collections
description: Understand how the collections system works
wip: false
template: post.jinja
tags:
  - documentation
  - posts
  - saaru
collections: # <--- here!
  - internals
```

Just mentioning the collection that each post belongs to allows you to access the collections and their posts in your templates as follows ->

```jinja
{% for collection in collections %}
  <p> <strong>{{collection}} </strong></p>
  {% for post in collections[collection] %}
    <p> <a href="{{post.link}}"> {{post.frontmatter.title}}</a> &rarr; {{post.frontmatter.description}}</p>
  {% endfor %}
{% endfor %}
```
