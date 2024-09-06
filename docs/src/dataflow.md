---
title: Augmenting Saaru Dataflow with DuckDB
description: Understand how the dataflow in Saaru works
wip: false
template: post.jinja
tags:
  - documentation
  - posts
  - saaru
collections:
  - internals
---

The main data stores of a Saaru application are a collection of HashMaps

1. `frontmatter_map` -> HashMap<String, AugmentedFrontMatter>
2. `tag_map` -> HashMap<String, Vec<ThinAugmentedFrontMatter>>
3. `collection_map` -> HashMap<String, Vec<ThinAugmentedFrontMatter>>

When the render pipeline is triggered, multiple things happen:

1. The Data Preprocessing Step
   1. The file content is read into memory
      1. Frontmatter is parsed
      2. The file content is parsed and stored in the Augmented frontmatter
         struct
      3. We create two copies of the augmented frontmatter struct, one for the
         tag index and one for the collection index
      4. We insert the augmented frontmatter struct into the main index, the
         `frontmatter_map`
      5. For each collection we find in the frontmatter, we insert a clone of
         that collection into the collection_map at the key of the collection
         name
      6. For each tag we find in the frontmatter, we insert a clone of that tag
         into the tag_map at the key of the tag
2. All Files are Rendered
   1. The `frontmatter_map` is iterated over, and the files are rendered
      sequentially
3. The Tags Pages are Rendered
   1. Within the tags folder, each tag gets a page which links to all the posts
      that have that tag
   2. There's a single page that has all the tags and their posts in it, whihch
      is also rendered
4. The static folder is copied as is

The Proposal at this point - the MVP Stage - is to

1. Replace all internal maps with DuckDB
2. Replace all references to these maps with Queries to DuckDB

Once this is complete, the plan is to let users create and reference arbitrary
collections via custom SQL queries on the data schema present, and use them as
part of the render workflow.
