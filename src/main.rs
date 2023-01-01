use crate::saaru::{SaaruArguments, SaaruInstance};
mod saaru;

fn main() {
    let args = SaaruArguments::new("/home/anirudh/projects/custom-ssg/example_source/".to_string());
    let mut instance = SaaruInstance::new(args);
    instance.set_template_environment();
    // instance.recursively_render_from_directory();
    // Alternate Render Pipeline
    instance.alternate_render_pipeline();
}
