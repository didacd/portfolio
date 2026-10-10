use std::{io, path::Path};

use serde::Deserialize;
use tokio::fs;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Profile {
    pub(crate) name: String,
    pub(crate) role: String,
    pub(crate) location: String,
    pub(crate) avatar: String,
    pub(crate) github_username: String,
    pub(crate) socials: Vec<Social>,
    pub(crate) work: Vec<WorkExperience>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Social {
    pub(crate) text: String,
    pub(crate) link: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WorkExperience {
    pub(crate) duration: String,
    pub(crate) company: String,
    pub(crate) title: String,
    pub(crate) description: String,
}

pub(crate) async fn load(path: &Path) -> io::Result<Profile> {
    let source = fs::read_to_string(path).await?;
    serde_json::from_str(&source).map_err(io::Error::other)
}
