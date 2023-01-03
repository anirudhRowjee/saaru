use crate::saaru::{SaaruArguments, SaaruInstance};
use std::time;

mod saaru;

fn main() {
    // TODO Make this a command line argument
    let start = time::Instant::now();
    let args = SaaruArguments::new("/home/anirudh/projects/custom-ssg/example_source/".to_string());
    let mut instance = SaaruInstance::new(args);
    instance.set_template_environment();
    instance.alternate_render_pipeline();
    let end = time::Instant::now();
    println!("Total Time Taken -> {:?}", end - start);
}
