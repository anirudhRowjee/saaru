use gray_matter::{engine::YAML, Matter};
use minijinja::{context, Environment, Source};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};

use std::borrow::BorrowMut;
use std::fs;
use std::path::PathBuf;

// This is the main implementation struct for Saaru
// TODO Derive clap parsing for this
#[derive(Debug)]
pub struct SaaruArguments {
    base_dir: PathBuf,
    template_dir: PathBuf,
    source_dir: PathBuf,
    static_dir: PathBuf,
    pub build_dir: PathBuf,
}

impl SaaruArguments {
    // TODO Test this
    pub fn new(base_dir: String) -> Self {
        let root_path = PathBuf::from(&base_dir);
        let mut template_path = PathBuf::from(&base_dir);
        let mut static_path = PathBuf::from(&base_dir);
        let mut content_path = PathBuf::from(&base_dir);
        let mut build_path = PathBuf::from(&base_dir);

        template_path.push("templates/");
        static_path.push("static");
        content_path.push("src");
        build_path.push("build");

        SaaruArguments {
            base_dir: root_path,
            template_dir: template_path,
            source_dir: content_path,
            static_dir: static_path,
            build_dir: build_path,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct FrontMatter {
    title: String,
    description: String,
    tags: Vec<String>,
    wip: bool,
}

// Runtime necessities of the Saaru application
pub struct SaaruInstance<'a> {
    pub template_env: Environment<'a>,
    // TODO Currently set to frontmatter YAML, see if you need to change this Via a config file later
    pub frontmatter_parser: Matter<YAML>,
    markdown_options: Options,
    arguments: SaaruArguments,
}

impl SaaruInstance<'_> {
    pub fn new(args: SaaruArguments) -> Self {
        // Prepare the Markdown Rendering Options
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

        // Set the source of the environment to be the template directory
        dbg!(&args);

        SaaruInstance {
            template_env: Environment::new(),
            frontmatter_parser: Matter::new(),
            markdown_options: options,
            arguments: args,
        }
    }

    pub fn set_template_environment(&mut self) {
        // TODO Replace this with environment which will read source
        self.template_env = Environment::new();
        self.template_env
            .set_source(Source::from_path(&self.arguments.template_dir));
        // dbg!(&self.arguments.template_dir);
        // dbg!(&self.template_env);
    }

    // Basic Implementation - Take in a file, render the HTML
    pub fn render_file(&mut self, filename: &str) -> String {
        // TODO Remove how you make this file
        let markdown_file_content = fs::read_to_string(filename).unwrap();
        let new_template = self.template_env.get_template("hello.jinja").unwrap();

        // Parse the frontmatter
        let parsed_frontmatter: FrontMatter = self
            .frontmatter_parser
            .parse(&markdown_file_content)
            .data
            .unwrap()
            .deserialize()
            .unwrap();

        // println!("Parsed Frontmatter: {:?}", parsed_frontmatter);

        // TODO remove the frontmatter once it's been parsed
        let cleaned_markdown = remove_frontmatter(&markdown_file_content);

        let parser = Parser::new_ext(&cleaned_markdown, self.markdown_options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        // println!("Parsed HTML => {:?}", html_output);

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
        write_html
    }

    pub fn write_html_to_file(self, output_filename: &str, input_html: String) {
        // Write the HTML rendered to a file
        fs::create_dir(&self.arguments.build_dir).unwrap();

        let mut output_path = self.arguments.build_dir.clone();
        output_path.push(output_filename);
        dbg!(&output_path);

        fs::File::create(&output_path).unwrap();
        dbg!(&output_path);

        fs::write(output_path, input_html).expect("Could not write!");
        println!("Successfully written to a file!");
    }
}

pub fn remove_frontmatter(markdown_file_content: &str) -> String {
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
#[cfg(test)]
mod tests {
    use crate::saaru::remove_frontmatter;

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

    #[test]
    fn test_frontmatter_cleaner_multiple_segment_tables() {
        let non_cleaned_string: &str =
            "---\nthis should not be there\n---\nthis is okay\n---\nthis should make it in\n---";
        let cleaned_string: &str = "this is okay\n---\nthis should make it in\n---";

        assert_eq!(
            cleaned_string.to_owned(),
            remove_frontmatter(non_cleaned_string)
        )
    }
}
