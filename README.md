# Saaru

Saaru is now an SSG Written in Rust. It uses Jinja Templates to render, along with Markdown to maintain content.

### Etymology
Saaru means Rasam, which is a type of spicy, thin lentil soup, often eaten with rice. This project is called Saaru because I like Saaru very much.

### Nonsense Formal explanation
SAARU -> StAtic Almanac Renderer and Unifier

## Feature map
- [x] Render a single markdown file to HTML
- [x] Render a single markdown file to HTML with a Jinja Template
- [ ] URGENT: Remove frontmatter from being rendered in the HTML
- [ ] Make templates readable from a single directory
- [ ] Run Pre-flight checks (check if templates dir exists, check if source dir exists, etc)
- [ ] Render a directory structure of Markdown and Jinja to a directory structure of HTML
- [ ] External CSS / Custom CSS injection
- [ ] Parallelized rendering (see HACK comments)
- [ ] Think about [Deep Data Merge](https://www.11ty.dev/docs/data/)
    - [ ] Think about single-tree-pass DDM Data Sourcing
    - [ ] Collect frontmatter data
- [ ] Web server + Live reload?
- [ ] tree-shaken rendering, only re-render what's changed?
    - [ ] Merkle Tree based hash checks?



### Old Plans

Bespoke is a proposed static site generator that uses Markdown for Content, Lisp for structuring, and HTML/CSS Combined with Templates for rendering.

Input -> description.el

```lisp

(setq books-template
    :title "Books"
    :description "Here are all my books"
    :description_md "./books_desc.md"
)

;; This is what my current blog's homepage would look like
(setq home-template
    :title "Home | Anirudh Rowjee"
    :description_md "./books_desc.md"
    :sublayout (
        ;; Assume these are defined somewhere
        header_template
        cta_submenu_template
        selected_posts_template
        footer
    )
)

(setq blog-template
    :title "Blog | Anirudh Rowjee"
    :description_md "./blog_desc.md"
    :sublayout (
        (header_template custom_param_1 custom_param_2)
        searchbar
        (content-grid 'posts)
    )
)


(site
    (site-metadata
       (author-name "Anirudh Rowjee")
       (domain-name "rowjee.com")
    )
    (content-map
        (home 'home-template)
        (blog 'blog-template)
        (about 'about-template)
        (projects 'projects-template)
        (books 'books-template)
    )
)
```
