use std::{io, path::PathBuf, sync::Arc};

use crate::github::GithubCache;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) content_dir: PathBuf,
    pub(crate) projects_dir: PathBuf,
    pub(crate) github: Arc<GithubCache>,
}

impl AppState {
    pub(crate) fn from_env() -> io::Result<Self> {
        let content_dir = std::env::var_os("CONTENT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("content/posts"));

        let projects_dir = std::env::var_os("PROJECTS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("content/projects"));

        Ok(Self {
            content_dir,
            projects_dir,
            github: Arc::new(GithubCache::new()?),
        })
    }
}
