use crate::saaru::{SaaruArguments, SaaruInstance};
mod saaru;

const LOGO: &str = r"
   ____
  / __/__ ____ _______ __
 _\ \/ _ `/ _ `/ __/ // /
/___/\_,_/\_,_/_/  \_,_/

";

fn main() {
    println!("{}", LOGO);

    let mut args =
        SaaruArguments::new("/home/anirudh/projects/custom-ssg/example_source/".to_string());

    let mut instance = SaaruInstance::new(args);
    instance.set_template_environment();

    let file_content =
        instance.render_file("/home/anirudh/projects/custom-ssg/example_source/src/index.md");
    instance.write_html_to_file("output.html", file_content)
}
