use std::path::{Path, PathBuf};



#[derive(Debug)]
pub struct SaaruArguments {
    pub base_dir: PathBuf,
    pub template_dir: PathBuf,
    pub source_dir: PathBuf,
    pub static_dir: PathBuf,
    pub build_dir: PathBuf,
}

impl SaaruArguments {
    pub fn new(mut base_dir: PathBuf) -> Self {
        log::info!("Initializing Arguments");

        base_dir = std::fs::canonicalize(base_dir).unwrap();

        let mut template_path = PathBuf::from(&base_dir);
        let mut static_path = PathBuf::from(&base_dir);
        let mut content_path = PathBuf::from(&base_dir);
        let mut build_path = PathBuf::from(&base_dir);

        template_path.push("templates/");
        static_path.push("static");
        content_path.push("src");
        build_path.push("build");

        log::info!("Initalized Arguments from Base Path {:?}", &base_dir);

        SaaruArguments {
            base_dir,
            template_dir: template_path,
            source_dir: content_path,
            static_dir: static_path,
            build_dir: build_path,
        }
    }
}
