use comrak::{markdown_to_html, ComrakOptions};
use gray_matter::{engine::YAML, Matter};
use minijinja::{context, value::Value, Environment, Source};
use walkdir::WalkDir;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::arguments::SaaruArguments;
use crate::frontmatter::{AugmentedFrontMatter, FrontMatter, ThinAugmentedFrontMatter};
use crate::utils::copy_recursively;

// This is the main implementation struct for Saaru

// Runtime necessities of the Saaru application
pub struct SaaruInstance<'a> {
    pub template_env: Environment<'a>,

    // TODO Currently set to frontmatter YAML, see if you need to change this Via a config file later
    pub frontmatter_parser: Matter<YAML>,
    markdown_options: ComrakOptions,
    pub arguments: SaaruArguments,
    // Runtime Data
    collection_map: HashMap<String, Vec<ThinAugmentedFrontMatter>>,
    tag_map: HashMap<String, Vec<ThinAugmentedFrontMatter>>,
    pub frontmatter_map: HashMap<String, AugmentedFrontMatter>,
    // Keep this default template
    default_template: String,
    // serialize and generate the default context ahead of time to have faster renders
    base_context: Value,
}

const LOGO: &str = r"
   ____
  / __/__ ____ _______ __
 _\ \/ _ `/ _ `/ __/ // /
/___/\_,_/\_,_/_/  \_,_/

A Static Site Generator for Fun and Profit
";

impl SaaruInstance<'_> {
    /*
     * functions for Saaru
     */

    pub fn new(args: SaaruArguments) -> Self {
        log::info!("{}", LOGO);
        log::info!("Printed Logo");

        let mut options = ComrakOptions::default();
        options.extension.front_matter_delimiter = Some("---".to_owned());
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        options.extension.tagfilter = true;
        options.extension.footnotes = true;
        options.extension.strikethrough = true;
        options.extension.description_lists = true;
        // options.extension.superscript = true;

        let default_template = args.json_content["metadata"]["templates"]["default"]
            .as_str()
            .unwrap()
            .to_string();
        log::info!("Default Jinja Template -> {:?}", &default_template);

        SaaruInstance {
            template_env: Environment::new(),
            frontmatter_parser: Matter::new(),
            markdown_options: options,
            arguments: args,

            // Data Merge
            collection_map: HashMap::new(),
            tag_map: HashMap::new(),
            frontmatter_map: HashMap::new(),
            base_context: context!(),
            default_template,
        }
    }

    pub fn validate_source_structure(&self) -> bool {
        // Check if the source directory structure is as it's supposed to be
        // TODO later validate for the right files existing
        self.arguments.source_dir.exists()
            && self.arguments.template_dir.exists()
            && self.arguments.static_dir.exists()
    }

    pub fn set_template_environment(&mut self) {
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

    pub fn get_relative_path_from_write_path(&self, write_path: &PathBuf) -> PathBuf {
        // Strip the base directory from the write path, giving you the build-local
        // Hyperlink you can drop in to the HTML to have valid links
        // Assumes input is coming from the get_write_path function
        let dir_path = write_path
            .strip_prefix(&self.arguments.build_dir)
            .unwrap()
            .to_path_buf();
        let mut relative = PathBuf::from("/");
        relative = relative.join(dir_path);
        log::info!("Stripped Relative Path -> {:?}", relative);
        relative
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

        let cleaned_markdown = markdown_file_content;
        let filename_str = filename.clone().display().to_string();

        let write_path = self.get_write_path(filename);
        let relative_build_path = self.get_relative_path_from_write_path(&write_path);

        let aug_fm_struct = AugmentedFrontMatter {
            file_content: cleaned_markdown.clone(),
            frontmatter: parsed_frontmatter.clone(),
            source_path: filename_str.clone(),
            write_path: write_path.display().to_string(),
            relative_build_path: relative_build_path.display().to_string(),
        };

        let tag_copy = aug_fm_struct.clone();
        let collection_copy = aug_fm_struct.clone();

        // Add the file to the tag map
        match &tag_copy.frontmatter.tags {
            Some(tag_list) => {
                for tag in tag_list {
                    self.tag_map
                        .entry(tag.to_string())
                        .and_modify(|list| {
                            list.push(ThinAugmentedFrontMatter::from(tag_copy.clone()))
                        })
                        .or_insert({
                            let mut new: Vec<ThinAugmentedFrontMatter> = Vec::with_capacity(100);
                            new.push(ThinAugmentedFrontMatter::from(tag_copy.clone()));
                            new
                        });
                }
            }
            None => {
                log::warn!("No Tags found in file {:?}", &filename_str);
            }
        }

        // Check if there's a collection defined for that page
        match &collection_copy.frontmatter.collections {
            Some(collection_list) => {
                for collection in collection_list {
                    self.collection_map
                        .entry(collection.to_string())
                        .and_modify(|list| {
                            list.push(ThinAugmentedFrontMatter::from(collection_copy.clone()))
                        })
                        .or_insert({
                            let mut new: Vec<ThinAugmentedFrontMatter> = Vec::with_capacity(100);
                            new.push(ThinAugmentedFrontMatter::from(collection_copy.clone()));
                            new
                        });
                }
            }
            None => {
                log::warn!("No Collections found in file {:?}", &filename_str);
            }
        }

        self.frontmatter_map.insert(filename_str, aug_fm_struct);
    }

    pub fn convert_markdown_to_html(&self, markdown: &String) -> String {
        let parser = markdown_to_html(markdown, &self.markdown_options);
        parser
    }

    pub fn render_file_from_frontmatter(
        &self,
        input_aug_frontmatter: &AugmentedFrontMatter,
    ) -> String {
        // Conver the Markdown to HTML
        let html_output = self.convert_markdown_to_html(&input_aug_frontmatter.file_content);

        // Fetch the Template
        let rendered_template = match &input_aug_frontmatter.frontmatter.template {
            Some(template_name) => self.template_env.get_template(&template_name).unwrap(),
            None => self
                .template_env
                .get_template(&self.default_template)
                .unwrap(),
        };

        // Render the template
        let rendered_final_html = rendered_template
            .render(context!(
                frontmatter => input_aug_frontmatter.frontmatter,
                postcontent => html_output,
                base => &self.base_context
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
            let html_content = self.render_file_from_frontmatter(&val);
            self.write_html_to_file(PathBuf::from(&val.write_path), html_content);
        }
    }

    pub fn render_individual_file(&mut self, path: &PathBuf) {
        log::info!("[LIVERELOAD] Processing file {:?}", path);
        self.preprocess_file_data(&path);

        let current_frontmatter = self
            .frontmatter_map
            .get(&path.display().to_string())
            .unwrap()
            .clone();
        log::info!(
            "[LIVERELOAD] Triggering HTML Conversion for file {:?}",
            path
        );
        let html_content = self.render_file_from_frontmatter(&current_frontmatter);

        log::info!("[LIVERELOAD] Writing to Destination for file {:?}", path);
        self.write_html_to_file(PathBuf::from(&current_frontmatter.write_path), html_content);
    }

    fn render_tags_pages(&self) {
        // A function to render all pages for tags
        let tag_index_template = self.template_env.get_template("tags.jinja").unwrap();

        let tag_individual_template = self.template_env.get_template("tags_page.jinja").unwrap();

        let base_tags_path = self.arguments.build_dir.clone().join("tags");

        // Render the index page
        let tags_index_rendered_html = tag_index_template
            .render(context!(
                base => &self.base_context
            ))
            .unwrap();
        self.write_html_to_file(base_tags_path.join("index.html"), tags_index_rendered_html);

        // Render a page for every single tag
        for (key, val) in &self.tag_map {
            let tags_index_rendered_html = tag_individual_template
                .render(context!(
                    tag => &key,
                    posts => &val,
                    base => &self.base_context
                ))
                .unwrap();
            self.write_html_to_file(
                base_tags_path.join(PathBuf::from(format!("{}.html", key))),
                tags_index_rendered_html,
            );
        }
    }

    fn copy_static_folder(&self) {
        // Copy over the static folder from the source directory to the
        // build directory
        let source_path = &self.arguments.static_dir;
        let destination_path = &self.arguments.build_dir;
        log::info!(
            "Beginnning static folder copy from {:?} to {:?}",
            source_path,
            destination_path
        );
        copy_recursively(source_path, destination_path).unwrap();
    }

    pub fn alternate_render_pipeline(&mut self) {
        // Full pipeline for rendering again
        // Stage 0: Validate the submitted folder structur
        // Stage 1: Preprocess all files, make all necessary directories
        // Stage 2: Render everything from the preprocessed map

        log::info!("[PREFLIGHT] Validating Input Directory");
        if !self.validate_source_structure() {
            panic!("The Provided Source Directory is malformed! Please follow the right format.")
        }

        log::info!("[PREFLIGHT] Checking for Build Directory");
        match fs::create_dir(&self.arguments.build_dir) {
            Ok(_) => log::info!("Build Directory Created Successfully"),
            Err(_) => log::warn!("Build Directory Already Exists!"),
        };

        log::info!("[LOG] Recursively Preprocessing All Files");
        for dir in WalkDir::new(&self.arguments.source_dir) {
            let entry = dir.unwrap();
            let local_path = entry.path();
            let metadata = fs::metadata(&local_path).unwrap();
            if metadata.is_dir()
                || local_path.extension().unwrap().to_str().unwrap() != &"md".to_string()
            {
                continue;
            }
            log::info!("Processing File {:?}", entry);
            self.preprocess_file_data(entry.path());
            log::info!("Finished Processing File {:?}", entry);
        }

        log::info!("Generating DDM Context...");
        self.base_context = context!(
        tags => &self.tag_map,
        collections => &self.collection_map,
        json => &self.arguments.json_content
        );

        log::info!("Rendering All Files...");
        self.render_all_files();
        log::info!("Rendering Tags");
        self.render_tags_pages();
        log::info!("Copying the static folder... ");
        self.copy_static_folder();
    }
}
