use crate::saaru::{SaaruArguments, SaaruInstance};
mod saaru;

fn main() {
    let args = SaaruArguments::new("/home/anirudh/projects/custom-ssg/example_source/".to_string());
    let mut instance = SaaruInstance::new(args);
    instance.set_template_environment();
    instance.alternate_render_pipeline();
}
