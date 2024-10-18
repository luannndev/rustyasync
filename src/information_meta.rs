use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ProjectMetaInformation {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) author: String,
    pub(crate) dependencies: Vec<DependencyInformation>,
}

#[derive(Serialize, Deserialize)]
pub struct DependencyInformation {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) remote: String,
}

impl ProjectMetaInformation {
    pub fn new(name: String, version: String, author: String) -> Self {
        Self {
            name,
            version,
            author,
            dependencies: vec![],
        }
    }
}

pub enum ProjectType {
    DefaultExecutable,
    DefaultLibrary,
}