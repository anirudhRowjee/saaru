// This is the main implementation struct for Saaru
//
// TODO Derive clap parsing for this
pub struct SaaruArguments {
    // Basic Arguments
    base_dir: String,
    template_dir: String,
    source_dir: String,
    static_dir: String,
}

// Runtime necessities of the Saaru application
pub struct SaaruInstance {
    // TODO Include Jinja Template Environment
    // TODO Include all frontmatter data
    name: String,
}

pub impl SaaruInstance {
    // Basic Implementation - Take in a file, render the HTML
    fn render_file(&mut self, filename: String) -> String {
        filename
    }

    // Function to parse the templates from the templates directory
    fn parse_templates(&mut self) {}
}

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

#[cfg(test)]
mod tests {

    use crate::saaru::SaaruInstance;

    #[test]
    fn test_frontmatter_cleaner() {
        let non_cleaned_string: &str = "---\nthis should not be there\n---\nthis is okay";
        let cleaned_string: &str = "this is okay";

        assert_eq!(
            cleaned_string.to_owned(),
            SaaruInstance::remove_frontmatter(non_cleaned_string)
        )
    }

    #[test]
    fn test_frontmatter_cleaner_multiple_segment() {
        let non_cleaned_string: &str =
            "---\nthis should not be there\n---\nthis is okay\n---\nthis should make it in";
        let cleaned_string: &str = "this is okay\n---\nthis should make it in";

        assert_eq!(
            cleaned_string.to_owned(),
            SaaruInstance::remove_frontmatter(non_cleaned_string)
        )
    }

    #[test]
    fn test_frontmatter_cleaner_multiple_segment_tables() {
        let non_cleaned_string: &str =
            "---\nthis should not be there\n---\nthis is okay\n---\nthis should make it in\n---";
        let cleaned_string: &str = "this is okay\n---\nthis should make it in\n---";

        assert_eq!(
            cleaned_string.to_owned(),
            SaaruInstance::remove_frontmatter(non_cleaned_string)
        )
    }
}
