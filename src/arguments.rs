use serde_json::Value;
use std::{fs::read_to_string, path::PathBuf};

#[derive(Debug)]
pub struct SaaruArguments {
    pub base_dir: PathBuf,
    pub template_dir: PathBuf,
    pub source_dir: PathBuf,
    pub static_dir: PathBuf,
    pub build_dir: PathBuf,
    pub json_content: Value,
    // Arguments for Live Reload and so on
    pub live_reload: bool,
    pub live_rerender: bool,
}

impl SaaruArguments {
    pub fn new(mut base_dir: PathBuf, live_reload: bool, live_rerender: bool) -> Self {
        log::info!("Initializing Arguments");

        base_dir = std::fs::canonicalize(base_dir).unwrap();

        let mut template_path = PathBuf::from(&base_dir);
        let mut static_path = PathBuf::from(&base_dir);
        let mut content_path = PathBuf::from(&base_dir);
        let mut build_path = PathBuf::from(&base_dir);
        let json_path = PathBuf::from(&base_dir).join(".saaru.json");

        template_path.push("templates/");
        static_path.push("static");
        content_path.push("src");
        build_path.push("build");
        log::info!("Initalized Arguments from Base Path {:?}", &base_dir);

        // Read the JSON
        // TODO Validate this
        let raw_json_content = match read_to_string(json_path) {
            Ok(content) => serde_json::from_str(&content).unwrap(),
            Err(_) => {
                // TODO better error handling here - check for another issue that
                // probably isn't the "didn't find file" error
                log::error!("Couldn't find .saaru.json in {:?}", &base_dir);
                log::warn!("Using default values!");
                serde_json::json!({
                  "metadata": {
                    "author": {
                      "name": "Author",
                      "one_line_desc": "hello, world!",
                      "twitter": "twitter.com/username",
                      "github": "github.com/username",
                    },
                    "templates": {
                        "default": "post.json",
                    }
                  }
                })
            }
        };
        let json_content = raw_json_content;
        log::info!("Finished Reading JSON Content -> {:?}", json_content);

        SaaruArguments {
            base_dir,
            template_dir: template_path,
            source_dir: content_path,
            static_dir: static_path,
            build_dir: build_path,
            json_content,
            live_reload,
            live_rerender,
        }
    }
}
