use gray_matter::{engine::YAML, Matter};
use minijinja::{context, Environment, Source};
use pulldown_cmark::{html, Options, Parser};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

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
        println!("[LOG] Initializing Arguments");
        let root_path = PathBuf::from(&base_dir);
        let mut template_path = PathBuf::from(&base_dir);
        let mut static_path = PathBuf::from(&base_dir);
        let mut content_path = PathBuf::from(&base_dir);
        let mut build_path = PathBuf::from(&base_dir);

        template_path.push("templates/");
        static_path.push("static");
        content_path.push("src");
        build_path.push("build");

        println!("[LOG] Initalized Arguments from Base Path {:?}", root_path);

        SaaruArguments {
            base_dir: root_path,
            template_dir: template_path,
            source_dir: content_path,
            static_dir: static_path,
            build_dir: build_path,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct FrontMatter {
    title: String,
    description: String,
    tags: Vec<String>,
    wip: bool,
    // This is the optional template string
    template: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AugmentedFrontMatter {
    frontmatter: FrontMatter,
    source_path: String,
    file_content: String,
    write_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThinAugmentedFrontMatter {
    frontmatter: FrontMatter,
    source_path: String,
    write_path: String,
}

impl From<AugmentedFrontMatter> for ThinAugmentedFrontMatter {
    fn from(old: AugmentedFrontMatter) -> Self {
        ThinAugmentedFrontMatter {
            frontmatter: old.frontmatter.clone(),
            source_path: old.source_path.clone(),
            write_path: old.write_path.clone(),
        }
    }
}

// Runtime necessities of the Saaru application
pub struct SaaruInstance<'a> {
    pub template_env: Environment<'a>,

    // TODO Currently set to frontmatter YAML, see if you need to change this Via a config file later
    pub frontmatter_parser: Matter<YAML>,
    markdown_options: Options,
    arguments: SaaruArguments,

    // Runtime Data
    collection_map: HashMap<String, Vec<FrontMatter>>,
    tag_map: HashMap<String, Vec<ThinAugmentedFrontMatter>>,
    frontmatter_map: HashMap<String, AugmentedFrontMatter>,
}

const LOGO: &str = r"
   ____
  / __/__ ____ _______ __
 _\ \/ _ `/ _ `/ __/ // /
/___/\_,_/\_,_/_/  \_,_/

A Static Site Generator for Fun and Profit
";

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

impl SaaruInstance<'_> {
    /*
     * functions for Saaru
     */

    pub fn new(args: SaaruArguments) -> Self {
        simple_logger::SimpleLogger::new().env().init().unwrap();

        println!("{}", LOGO);
        log::info!("Printed Logo");

        log::info!("Initialized Logger");

        SaaruInstance {
            template_env: Environment::new(),
            frontmatter_parser: Matter::new(),
            markdown_options: Options::all(),
            arguments: args,

            // Data Merge
            collection_map: HashMap::new(),
            tag_map: HashMap::new(),
            frontmatter_map: HashMap::new(),
        }
    }

    pub fn set_template_environment(&mut self) {
        // TODO Replace this with environment which will read source
        self.template_env = Environment::new();
        self.template_env
            .set_source(Source::from_path(&self.arguments.template_dir));
        log::info!("Initialized Template Environment");
    }

    pub fn get_write_path(&self, entry_path: &Path) -> PathBuf {
        // Generate the final write path ->
        // Input: src/posts/a.md
        // Output: build/posts/a.html

        let mut write_path = entry_path.to_path_buf();

        write_path = write_path
            .strip_prefix(&self.arguments.source_dir)
            .unwrap()
            .to_path_buf();

        write_path.set_extension("html");

        // Append the write path into the base directory
        let final_write_path = self.arguments.build_dir.join(&write_path);
        final_write_path
    }

    pub fn preprocess_file_data(&mut self, filename: &Path) {
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
        let filename_str = filename.clone().display().to_string();

        let aug_fm_struct = AugmentedFrontMatter {
            file_content: cleaned_markdown.clone(),
            frontmatter: parsed_frontmatter.clone(),
            source_path: filename_str.clone(),
            write_path: self.get_write_path(filename).to_str().unwrap().to_string(),
        };

        let tag_copy = aug_fm_struct.clone();
        let collection_copy = aug_fm_struct.clone();

        for tag in &tag_copy.frontmatter.tags {
            self.tag_map
                .entry(tag.to_string())
                .and_modify(|list| list.push(ThinAugmentedFrontMatter::from(tag_copy.clone())))
                .or_insert(vec![ThinAugmentedFrontMatter::from(tag_copy.clone())]);
        }

        self.frontmatter_map.insert(filename_str, aug_fm_struct);

        // Load the Tag Map

        // TODO Insert into tag and collection maps, respectively
    }

    pub fn render_file_from_frontmatter(
        &self,
        input_aug_frontmatter: AugmentedFrontMatter,
    ) -> String {
        let parser = Parser::new_ext(&input_aug_frontmatter.file_content, self.markdown_options);
        let mut html_output = String::new();

        html::push_html(&mut html_output, parser);

        // Render the template
        let rendered_template = match &input_aug_frontmatter.frontmatter.template {
            Some(template_name) => self.template_env.get_template(&template_name).unwrap(),
            None => {
                panic!("Could not find template")
            }
        };

        let rendered_final_html = rendered_template
            .render(context!(
                name => "Anirudh",
                users => vec!["a", "b", "c"],
                frontmatter => input_aug_frontmatter.frontmatter,
                postcontent => html_output,
                tags => &self.tag_map,
            ))
            .unwrap();

        // Copy just for fun
        rendered_final_html
    }

    pub fn write_html_to_file(&self, output_filename: PathBuf, input_html: String) {
        // Create the file and folder if it doesn't exist, write it to disk

        // Generate the output path from the build directory and the given output filename
        let mut output_path = self.arguments.build_dir.clone();
        output_path.push(output_filename);

        // Create all the necessary directories that need to be created
        let current_prefix = output_path.parent().unwrap();

        fs::create_dir_all(current_prefix).unwrap();

        // Create the file itself
        fs::File::create(&output_path).unwrap();

        // Write to the file
        fs::write(&output_path, input_html).expect("Could not write!");
        log::info!("SUCCESS: Wrote to {:?}", &output_path);
    }

    pub fn render_all_files(&self) {
        // Render the entire map
        for (key, val) in &self.frontmatter_map {
            // Key => Path
            // Value => AugmentedFrontMatter
            log::info!("Rendering file {:?} to Path {:?}", key, val.write_path);

            let new_val = val.clone();
            let html_content = self.render_file_from_frontmatter(new_val);
            self.write_html_to_file(PathBuf::from(&val.write_path), html_content);
        }
    }

    pub fn alternate_render_pipeline(&mut self) {
        // Full pipeline for rendering again
        // Stage 1: Preprocess all files, make all necessary directories
        // Staege 2: Render everything from the preprocessed map

        log::info!("[PREFLIGHT] Checking for Build Directory");
        match fs::create_dir(&self.arguments.build_dir) {
            Ok(_) => log::info!("Build Directory Created Successfully"),
            Err(_) => log::warn!("Build Directory Already Exists!"),
        };

        log::info!("[LOG] Recursively Preprocessing All Files");
        for dir in WalkDir::new(&self.arguments.source_dir) {
            let entry = dir.unwrap();
            let metadata = fs::metadata(entry.path()).unwrap();
            // Skip if directory
            if metadata.is_dir() {
                continue;
            }

            log::info!("Processing File {:?}", entry);
            let entry_path = entry.path();
            let final_write_path = self.get_write_path(entry_path);
            log::info!("Generated Write Path {:?}", final_write_path);

            self.preprocess_file_data(entry_path);
            log::info!("Finished Processing File {:?}", entry);
        }

        // Print out the tag map
        println!("Tag Map -> {:?}", self.tag_map);

        log::info!("Rendering Stage");
        self.render_all_files();
    }
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

    #[test]
    fn test_frontmatter_cleaner_does_not_harm_tables() {
        let non_cleaned_string: &str =
            "---\nthis should not be there\n---\nthis is okay\n---\nthis should make it in\n---\n | Header  | Another Header |\n | ------- | -------------- |\n | field 1 | value one      | ";
        let cleaned_string: &str = "this is okay\n---\nthis should make it in\n---\n | Header  | Another Header |\n | ------- | -------------- |\n | field 1 | value one      | ";

        assert_eq!(
            cleaned_string.to_owned(),
            remove_frontmatter(non_cleaned_string)
        )
    }
}
