use std::fs;

use gray_matter::{engine::YAML, Matter};
use minijinja::{context, Environment};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};

/*
 * TODO Write operation to parse directory tree and scan for - templates, source files, etc
 *
 * For example, consider the following directory -
 * test_src
 * - index.md
 * - post1.md
 * - post2.md
 * - post3.md
 *
 * The rendered output would look something like -
 * build_src
 * - index.html
 * - post1.html
 * - post2.html
 * - post3.html
 *
 * HACK Consider how you'll add an external template for all content, nest everything in that
 * HACK Consider globalising templates
 * HACK Consider doing one full directory tree pass to get all information at once
 * HACK Consider rendering everything in parallel with a single 10-thread pool
 * HACK Consider parser modification/plugins for admonitions
 */

#[derive(Serialize, Deserialize, Debug)]
struct FrontMatter {
    title: String,
    description: String,
    tags: Vec<String>,
    wip: bool,
}

fn main() {
    // Initialize The Jinja Renderer
    let mut env = Environment::new();
    env.add_template("Hello", include_str!("./test_template.jinja"))
        .unwrap();
    let new_template = env.get_template("Hello").unwrap();

    // Initialize the frontmatter parser
    let mut matter: Matter<YAML> = Matter::new();

    // Initialize the markdown parser
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    // Parse the frontmatter
    let parsed_frontmatter: FrontMatter = matter
        .parse(include_str!("./test_content.md"))
        .data
        .unwrap()
        .deserialize()
        .unwrap();
    println!("Parsed Frontmatter: {:?}", parsed_frontmatter);

    // Parse the markdown content in itself
    let markdown_file_content = include_str!("./test_content.md");
    let parser = Parser::new_ext(markdown_file_content, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    println!("Parsed HTML => {:?}", html_output);
    fs::write("./raw_html.html", html_output.clone()).expect("could not write");

    // Render the template
    let rendered_final_html = new_template
        .render(context!(
            name => "Anirudh",
            users => vec!["a", "b", "c"],
            frontmatter => parsed_frontmatter,
            postcontent => html_output
        ))
        .unwrap();
    let write_html = rendered_final_html.clone();

    // Write the HTML rendered to a file
    fs::write("./index.html", write_html).expect("Could not write!");
    println!("Successfully written to a file!");

    println!("Hello, world!");
}
