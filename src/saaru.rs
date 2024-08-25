use axum::routing::IntoMakeService;
use axum::{http, routing::get_service, Router};
use comrak::{markdown_to_html, ComrakOptions};
use crossbeam::channel::unbounded;
use gray_matter::{engine::YAML, Matter};
use minijinja::{context, path_loader, value::Value, Environment};
use notify::event::AccessKind;
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use notify::{Error, Event};
use tower::layer::util::Stack;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_livereload::LiveReloadLayer;
use walkdir::WalkDir;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::thread;
use std::time;

use crate::arguments::SaaruArguments;
use crate::frontmatter::{AugmentedFrontMatter, FrontMatter, ThinAugmentedFrontMatter};
use crate::utils::copy_recursively;

// This is the main implementation struct for Saaru
#[derive(Debug)]
pub enum SaaruEvent {
    FileChanged(Result<Event, Error>),
    FileReRenderCompleted,
}

// Runtime necessities of the Saaru application
pub struct SaaruInstance {
    pub template_env: Environment<'static>,

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

    // Threadpool thread count
    parallel_render_threads: usize,
    // Threads
    render_channel_producer: crossbeam::channel::Sender<Option<(String, AugmentedFrontMatter)>>,
    render_channel_consumer: crossbeam::channel::Receiver<Option<(String, AugmentedFrontMatter)>>,
}

const LOGO: &str = r"
   ____
  / __/__ ____ _______ __
 _\ \/ _ `/ _ `/ __/ // /
/___/\_,_/\_,_/_/  \_,_/

A Static Site Generator for Fun and Profit
";

impl SaaruInstance {
    /*
     * functions for Saaru
     */

    pub fn new(args: SaaruArguments) -> Self {
        log::info!("{}", LOGO);
        log::info!("Printed Logo");

        let mut options = ComrakOptions::default();
        options.extension.front_matter_delimiter = Some("---".to_owned());

        // Allow raw HTML!
        options.render.unsafe_ = true;
        options.extension.table = true;
        options.extension.autolink = true;
        options.extension.tasklist = true;
        // options.extension.tagfilter = true;
        options.extension.footnotes = true;
        options.extension.strikethrough = true;
        options.extension.description_lists = true;
        // options.extension.superscript = true;
        // options.extension.tagfilter = true;

        // TODO see where this can fail
        let default_template = args.json_content["metadata"]["templates"]["default"]
            .as_str()
            .unwrap()
            .to_string();
        log::info!("Default Jinja Template -> {:?}", &default_template);

        let (tx, rx) = unbounded::<Option<(String, AugmentedFrontMatter)>>();

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
            // TODO Read from config later
            parallel_render_threads: 10,
            render_channel_producer: tx,
            render_channel_consumer: rx,
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
            .set_loader(path_loader(&self.arguments.template_dir));
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
        let file = File::open(filename).unwrap();
        let mut reader = BufReader::new(file);
        let mut markdown_file_content = String::new();
        reader.read_to_string(&mut markdown_file_content).unwrap();

        // Parse the frontmatter
        let parsed_frontmatter: FrontMatter = self
            .frontmatter_parser
            .parse(&markdown_file_content)
            .data
            .unwrap()
            .deserialize()
            .unwrap();

        let cleaned_markdown = markdown_file_content;
        let filename_str = filename.display().to_string();

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

        self.frontmatter_map.insert(filename_str, aug_fm_struct);

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
                // log::warn!("No Tags found in file {:?}", &filename_str);
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
                // log::warn!("No Collections found in file {:?}", &filename_str);
            }
        }
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
        let file = fs::File::create(&output_path).unwrap();

        // create a buffered writer
        let mut writer = BufWriter::new(file);
        let input_bytes = input_html.as_bytes();

        // Write to the file
        writer.write_all(input_bytes).expect("Could not write!");
        log::info!("SUCCESS: Wrote to {:?}", &output_path);
    }

    pub fn render_all_files(&self) {
        std::thread::scope(|scope| {
            scope.spawn(|| {
                // Render the entire map
                for (key, val) in &self.frontmatter_map {
                    // Key => Path
                    let key = key.clone();
                    // Value => AugmentedFrontMatter
                    let value = val.clone();

                    log::info!("Rendering file {:?} to Path {:?}", key, value.write_path);
                    self.render_channel_producer
                        .send(Some((key, value)))
                        .unwrap();
                }
                // terminate all threads
                for _ in 0..self.parallel_render_threads {
                    self.render_channel_producer.send(None).unwrap();
                }
            });

            for x in 0..self.parallel_render_threads {
                // TODO Figure out clean exit later
                log::info!("Launching Thread ID {x}");
                scope.spawn(|| loop {
                    match self.render_channel_consumer.recv() {
                        Ok(Some(work)) => {
                            let key = work.0;
                            let val = work.1;
                            log::info!("Rendering file {:?} to Path {:?}", key, val.write_path);
                            let html_content = self.render_file_from_frontmatter(&val);
                            self.write_html_to_file(PathBuf::from(&val.write_path), html_content);
                        }
                        Ok(None) => {
                            log::info!("Terminating thread...");
                            break;
                        }
                        Err(e) => panic!("gone bro {}", e),
                    }
                });
                // self.threads.push(join_handle);
            }
        });
    }

    pub fn render_individual_file(&mut self, path: &PathBuf) {
        log::info!("[LIVERELOAD] Processing file {:?}", path);
        self.preprocess_file_data(&path);

        let current_frontmatter = self
            .frontmatter_map
            .get(&path.display().to_string())
            .unwrap()
            .clone();
        log::debug!(
            "[LIVERELOAD] Triggering HTML Conversion for file {:?}",
            path
        );
        let html_content = self.render_file_from_frontmatter(&current_frontmatter);

        log::debug!("[LIVERELOAD] Writing to Destination for file {:?}", path);
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

    // pub fn initialize_rendering_threadpool(&mut self) {
    //     // Spawn the threads
    //     // Create the channel that will hold the rendering context jobs
    //     // in each thread, setup the listener on that channel that will perform the rendering in
    //     // parallel
    //     std::thread::scope(|scope| {
    //         for _x in 0..self.parallel_render_threads {
    //             // TODO Figure out clean exit later
    //             scope.spawn(|| {
    //                 println!("Launching thread");
    //                 loop {
    //                     match self.render_channel_consumer.recv() {
    //                         Ok(work) => {
    //                             let key = work.0;
    //                             let val = work.1;
    //                             log::info!("Rendering file {:?} to Path {:?}", key, val.write_path);
    //                             let html_content = self.render_file_from_frontmatter(&val);
    //                             self.write_html_to_file(
    //                                 PathBuf::from(&val.write_path),
    //                                 html_content,
    //                             );
    //                         }
    //                         Err(e) => panic!("gone bro {}", e),
    //                     }
    //                 }
    //             });
    //             // self.threads.push(join_handle);
    //         }
    //     });
    // }

    pub fn render_pipeline(&mut self) {
        // Full pipeline for rendering again
        // Stage 0: Validate the submitted folder structur
        // Stage 1: Preprocess all files, make all necessary directories
        // Stage 2: Render everything from the preprocessed map

        log::debug!("[PREFLIGHT] Validating Input Directory");
        if !self.validate_source_structure() {
            panic!("The Provided Source Directory is malformed! Please follow the right format.")
        }

        log::debug!("[PREFLIGHT] Checking for Build Directory");
        match fs::create_dir(&self.arguments.build_dir) {
            Ok(_) => log::info!("Build Directory Created Successfully"),
            Err(_) => log::warn!("Build Directory Already Exists!"),
        };

        log::debug!("[LOG] Recursively Preprocessing All Files");
        for dir in WalkDir::new(&self.arguments.source_dir) {
            let entry = dir.unwrap();
            let local_path = entry.path();
            let metadata = fs::metadata(&local_path).unwrap();
            if metadata.is_dir()
                || local_path.extension().unwrap().to_str().unwrap() != &"md".to_string()
            {
                continue;
            }
            log::debug!("Processing File {:?}", entry);
            self.preprocess_file_data(entry.path());
            log::debug!("Finished Processing File {:?}", entry);
        }

        log::debug!("Generating DDM Context...");
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

    pub fn orchestrator(mut self) {
        // Launch Point for the Saaru Orchestrator
        log::info!("starting the orchestrator");
        let (tx, rx) = unbounded::<SaaruEvent>();

        // Make Separate Watch Dirs for each reloadable segment
        let watch_dir = self.arguments.source_dir.as_path();
        let static_watch_dir = self.arguments.static_dir.as_path();
        // let template_watch_dir = self.arguments.template_dir.as_path().clone();

        // Initialize Reloader
        let reload_layer = LiveReloadLayer::new();
        let reloader = reload_layer.reloader();

        let live_reload = self.arguments.live_reload.clone();
        let _live_rerender = self.arguments.live_rerender.clone();

        let watcher_sender = tx.clone();
        let static_watcher_sender = tx.clone();
        // let template_watcher_sender = tx.clone();

        let build_dir = self.arguments.build_dir.clone();

        // Setup the watcher (somehow works on a parallel thread?)
        let mut content_watcher = RecommendedWatcher::new(
            move |res: Result<Event, Error>| {
                watcher_sender.send(SaaruEvent::FileChanged(res)).unwrap();
            },
            Config::default(),
        )
        .unwrap();
        content_watcher
            .watch(watch_dir, RecursiveMode::Recursive)
            .unwrap();

        // Staticfile watcher
        let mut static_watcher = RecommendedWatcher::new(
            move |res: Result<Event, Error>| {
                static_watcher_sender
                    .send(SaaruEvent::FileChanged(res))
                    .unwrap();
            },
            Config::default(),
        )
        .unwrap();
        static_watcher
            .watch(static_watch_dir, RecursiveMode::Recursive)
            .unwrap();

        // // Template watcher
        // let mut template_watcher = RecommendedWatcher::new(
        //     move |res: Result<Event, Error>| {
        //         template_watcher_sender
        //             .send(SaaruEvent::FileChanged(res))
        //             .unwrap();
        //     },
        //     Config::default(),
        // )
        // .unwrap();
        // template_watcher
        //     .watch(template_watch_dir, RecursiveMode::Recursive)
        //     .unwrap();

        // Setup the listener (and now, orchestrator)
        let listener_thread = std::thread::spawn(move || {
            let listener = rx.clone();
            let sender = tx.clone();
            // Listen to all the events on the wire
            for x in listener {
                match x {
                    SaaruEvent::FileChanged(filechangeevent) => {
                        match filechangeevent {
                            Ok(event) => {
                                match event.kind {
                                    // Watch for files getting written again
                                    notify::EventKind::Access(AccessKind::Close(_metadata)) => {
                                        log::info!(
                                            "[LIVERELOAD] Re-processing file {:?}",
                                            &event.paths[0]
                                        );
                                        let start = time::Instant::now();

                                        // TODO Check if it's a static file, if so, copy over
                                        match &event.paths[0].extension().unwrap().to_str().unwrap()
                                        {
                                            &"md" => {
                                                log::info!("Changed Markdown File -> Re-rendering individual file");
                                                self.borrow_mut()
                                                    .render_individual_file(&event.paths[0]);
                                            }
                                            _ => {
                                                // Check if the re-render is from the static files
                                                // or if it's a template

                                                if &event.paths[0]
                                                    .starts_with(&self.arguments.static_dir)
                                                    == &true
                                                {
                                                    log::info!("Static File Changed. Skipping re-render, recopying static folder");
                                                    // Copy static folder again
                                                    self.borrow_mut().copy_static_folder();
                                                } else {
                                                    // Something
                                                    log::info!("Non-Static File Changed. Re-Rendering entire website.");

                                                    // Invalidate the frontmatter
                                                    log::warn!("Invalidating Collections Map");
                                                    self.borrow_mut().collection_map =
                                                        HashMap::new();
                                                    log::warn!("Invalidating Tag Map");
                                                    self.borrow_mut().tag_map = HashMap::new();
                                                    log::warn!("Invalidating Frontmatter Map");
                                                    self.borrow_mut().frontmatter_map =
                                                        HashMap::new();
                                                    log::info!("Triggering Full Site Re-Render");
                                                    self.borrow_mut().render_pipeline();
                                                }
                                            }
                                        }

                                        let end = time::Instant::now();
                                        log::info!(
                                            "File {:?} re-rendered in {:?}",
                                            event.paths[0],
                                            end - start
                                        );

                                        // TODO Trigger reload!
                                        sender.send(SaaruEvent::FileReRenderCompleted).unwrap();
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => log::error!("[LIVERELOAD] {:?}", e),
                        }
                    }
                    SaaruEvent::FileReRenderCompleted => {
                        // TODO implment the reload signal
                        log::info!("Recieved Re-Render Completion Event");
                        reloader.reload();
                    }
                }
            }
        });

        if live_reload {
            let server_thread = thread::spawn(move || {
                // Initialize Web Server
                let app = Router::new()
                    .nest_service("/", serve_dir(&build_dir))
                    .layer(reload_layer)
                    .layer(no_cache_layer())
                    .into_make_service();

                // start the web server
                let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 3000);
                start_server(&addr, app).unwrap();
            });

            server_thread.join().unwrap();
        }
        listener_thread.join().unwrap();
    }
}

// Attrib - https://github.com/leotaku/tower-livereload/blob/master/examples/livehttpd/src/main.rs
fn serve_dir(path: &std::path::Path) -> axum::routing::MethodRouter {
    get_service(ServeDir::new(path)).handle_error(|error| async move {
        (
            http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", error),
        )
    })
}
type Srhl = SetResponseHeaderLayer<http::HeaderValue>;

fn no_cache_layer() -> Stack<Srhl, Stack<Srhl, Srhl>> {
    Stack::new(
        SetResponseHeaderLayer::overriding(
            http::header::CACHE_CONTROL,
            http::HeaderValue::from_static("no-cache, no-store, must-revalidate"),
        ),
        Stack::new(
            SetResponseHeaderLayer::overriding(
                http::header::PRAGMA,
                http::HeaderValue::from_static("no-cache"),
            ),
            SetResponseHeaderLayer::overriding(
                http::header::EXPIRES,
                http::HeaderValue::from_static("0"),
            ),
        ),
    )
}

#[tokio::main]
async fn start_server(
    addr: &SocketAddr,
    app: IntoMakeService<Router>,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Serving on: http://{}/", addr);
    axum::Server::try_bind(&addr)?.serve(app).await?;
    Ok(())
}
