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

fn remove_frontmatter(markdown_file_content: &str) -> String {
    // This is a destructive write, we don't expect parallel ownership
    // of this anywhere

    let mut encounter_count = 0;
    let mut removal_complete = false;

    let in_frontmatter_block = markdown_file_content
        .to_string()
        .split("\n")
        .map(|segment| {
            if !removal_complete {
                if segment == "---" {
                    encounter_count += 1
                }
                if encounter_count % 2 == 0 {
                    removal_complete = true;
                }
                ""
            } else {
                segment
            }
        })
        .fold(String::new(), |mut a, b| -> String {
            // don't push a newline in these cases
            if a != "" && b != "" {
                a.push_str("\n");
            }
            a.push_str(&b.to_owned());
            a
        });
    in_frontmatter_block
}

fn main() {
    // Initialize The Jinja Renderer
    let mut env = Environment::new();
    env.add_template("Hello", include_str!("./test_template.jinja"))
        .unwrap();
    let new_template = env.get_template("Hello").unwrap();

    // Initialize the frontmatter parser
    let matter: Matter<YAML> = Matter::new();

    // Initialize the markdown parser
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);

    // TODO Remove how you make this file
    let markdown_file_content = include_str!("./test_content.md");

    // Parse the frontmatter
    let parsed_frontmatter: FrontMatter = matter
        .parse(markdown_file_content)
        .data
        .unwrap()
        .deserialize()
        .unwrap();

    println!("Parsed Frontmatter: {:?}", parsed_frontmatter);

    // TODO remove the frontmatter once it's been parsed
    let cleaned_markdown = remove_frontmatter(markdown_file_content);

    let parser = Parser::new_ext(&cleaned_markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    println!("Parsed HTML => {:?}", html_output);

    // Render the template
    let rendered_final_html = new_template
        .render(context!(
            name => "Anirudh",
            users => vec!["a", "b", "c"],
            frontmatter => parsed_frontmatter,
            postcontent => html_output
        ))
        .unwrap();

    // Copy just for fun
    let write_html = rendered_final_html.clone();
    // Write the HTML rendered to a file
    fs::write("./index.html", write_html).expect("Could not write!");
    println!("Successfully written to a file!");
}

#[cfg(test)]
mod tests {
    use crate::remove_frontmatter;

    #[test]
    fn test_frontmatter_cleaner() {
        let non_cleaned_string: &str = "---\nthis should not be there\n---\nthis is okay";
        let cleaned_string: &str = "this is okay";

        assert_eq!(
            cleaned_string.to_owned(),
            remove_frontmatter(non_cleaned_string)
        )
    }

    #[test]
    fn test_frontmatter_cleaner_multiple_segment() {
        let non_cleaned_string: &str =
            "---\nthis should not be there\n---\nthis is okay\n---\nthis should make it in";
        let cleaned_string: &str = "this is okay\n---\nthis should make it in";

        assert_eq!(
            cleaned_string.to_owned(),
            remove_frontmatter(non_cleaned_string)
        )
    }
}
