use std::path::Path;

use axum::{extract::State, http::StatusCode, response::Html};
use dioxus_mdx::strip_leading_h1;
use tokio::fs;

use crate::{content, render, state::AppState};

use super::internal_error;

pub(super) async fn index(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let (source, profile, posts, projects) = tokio::try_join!(
        fs::read_to_string("content/about.md"),
        content::profile::load(Path::new("content/profile.json")),
        content::list_entries(&state.content_dir),
        content::list_entries(&state.projects_dir),
    )
    .map_err(internal_error)?;
    let (_, body) = content::parse_frontmatter(&source).map_err(internal_error)?;
    let github = state.github.get(&profile.github_username).await;

    Ok(Html(render::home(
        strip_leading_h1(body),
        &profile,
        &posts,
        &projects,
        github.as_ref(),
    )))
}
