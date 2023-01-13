use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FrontMatter {
    pub title: String,
    pub description: String,
    pub date: Option<String>,
    pub tags: Option<Vec<String>>,
    pub collections: Option<Vec<String>>,
    pub wip: Option<bool>,
    pub template: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AugmentedFrontMatter {
    pub frontmatter: FrontMatter,
    pub source_path: String,
    pub file_content: String,
    pub write_path: String,
    pub relative_build_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ThinAugmentedFrontMatter {
    pub frontmatter: FrontMatter,
    pub source_path: String,
    pub write_path: String,
    pub link: String,
}

impl From<AugmentedFrontMatter> for ThinAugmentedFrontMatter {
    fn from(old: AugmentedFrontMatter) -> Self {
        ThinAugmentedFrontMatter {
            frontmatter: old.frontmatter,
            source_path: old.source_path,
            write_path: old.write_path,
            link: old.relative_build_path,
        }
    }
}
