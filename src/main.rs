use crate::arguments::SaaruArguments;
use crate::saaru::SaaruInstance;
use clap::Parser;
use std::path::PathBuf;
use std::time;

mod arguments;
mod frontmatter;
mod saaru;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// This is the base path
    #[arg(short, long)]
    base_path: PathBuf,
}

fn main() {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    log::info!("Initialized Logger");

    let commandline_arguments = Arguments::parse();
    log::info!("Command Line Arguments -> {:?}", &commandline_arguments);

    let args = SaaruArguments::new(commandline_arguments.base_path);
    let mut instance = SaaruInstance::new(args);

    let start = time::Instant::now();
    instance.set_template_environment();
    instance.alternate_render_pipeline();
    let end = time::Instant::now();
    println!("Total Time Taken -> {:?}", end - start);
}
