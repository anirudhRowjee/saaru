use gray_matter::{engine::YAML, Matter};
use minijinja::{context, Environment, Source};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

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
    // This is the optional template string
    template: Option<String>,
}

// Runtime necessities of the Saaru application
pub struct SaaruInstance<'a> {
    pub template_env: Environment<'a>,
    // TODO Currently set to frontmatter YAML, see if you need to change this Via a config file later
    pub frontmatter_parser: Matter<YAML>,
    markdown_options: Options,
    arguments: SaaruArguments,
}

const LOGO: &str = r"
   ____
  / __/__ ____ _______ __
 _\ \/ _ `/ _ `/ __/ // /
/___/\_,_/\_,_/_/  \_,_/

A Static Site Generator for Fun and Profit
";

impl SaaruInstance<'_> {
    pub fn new(args: SaaruArguments) -> Self {
        // Prepare the Markdown Rendering Options
        println!("{}", LOGO);
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_TASKLISTS);

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
    }

    pub fn render_file(&mut self, filename: &str) -> String {
        let markdown_file_content = fs::read_to_string(filename).unwrap();

        // Parse the frontmatter
        let parsed_frontmatter: FrontMatter = self
            .frontmatter_parser
            .parse(&markdown_file_content)
            .data
            .unwrap()
            .deserialize()
            .unwrap();

        let cleaned_markdown = remove_frontmatter(&markdown_file_content);

        let parser = Parser::new_ext(&cleaned_markdown, self.markdown_options);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        // Render the template
        let rendered_template = match &parsed_frontmatter.template {
            Some(template_name) => self.template_env.get_template(&template_name).unwrap(),
            None => {
                panic!("Could not find template")
            }
        };

        let rendered_final_html = rendered_template
            .render(context!(
                name => "Anirudh",
                users => vec!["a", "b", "c"],
                frontmatter => parsed_frontmatter,
                postcontent => html_output
            ))
            .unwrap();

        // Copy just for fun
        rendered_final_html
    }

    pub fn write_html_to_file(&self, output_filename: PathBuf, input_html: String) {
        let mut output_path = self.arguments.build_dir.clone();
        output_path.push(output_filename);
        // dbg!(&output_path);

        fs::File::create(&output_path).unwrap();
        // dbg!(&output_path);

        fs::write(&output_path, input_html).expect("Could not write!");
        println!("Successfully written to file => {:?}!", &output_path);
    }

    pub fn render_dir(&mut self) {
        // Write the HTML rendered to a file
        match fs::create_dir(&self.arguments.build_dir) {
            Ok(_) => println!("Build Directory Created Successfully"),
            Err(_) => println!("Build Directory Already Exists!"),
        };

        // Run the entire render operation in a directory structure
        // Walk the source directory
        for dir in WalkDir::new(&self.arguments.source_dir) {
            let entry = dir.unwrap();

            let metadata = fs::metadata(entry.path()).unwrap();
            if metadata.is_dir() {
                continue;
            }

            let entry_path = entry.path();

            // Make the write path
            let mut write_path = entry_path.to_path_buf();
            // println!("Write path before prefix strip -> {:?}", &write_path);

            write_path = write_path
                .strip_prefix(&self.arguments.source_dir)
                .unwrap()
                .to_path_buf();

            // println!(
            //     "Write Path before prepending the base directory -> {:?}",
            //     write_path
            // );

            write_path.set_extension("html");

            // Append the write path into the base directory
            let final_write_path = self.arguments.build_dir.join(&write_path);
            // println!(
            //     "Processing {:?} and writing to {:?}",
            //     entry_path, final_write_path
            // );

            // Create the final write path if it doesn't exist
            // TODO See where is the best place to put this
            let current_prefix = final_write_path.parent().unwrap();
            fs::create_dir_all(current_prefix).unwrap();

            let file_content = self.render_file(entry_path.to_str().unwrap());
            self.write_html_to_file(final_write_path, file_content);
        }
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