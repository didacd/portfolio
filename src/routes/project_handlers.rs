use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use dioxus_mdx::strip_leading_h1;

use crate::{content, render, state::AppState};

use super::internal_error;

pub(super) async fn index(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let projects = content::list_entries(&state.projects_dir)
        .await
        .map_err(internal_error)?;
    Ok(Html(render::project_list(&projects)))
}

pub(super) async fn show(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let Some((project, source)) = content::find_entry(&state.projects_dir, &slug)
        .await
        .map_err(internal_error)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };
    let (_, body) = content::parse_frontmatter(&source).map_err(internal_error)?;
    Ok(Html(render::project_detail(
        &project,
        strip_leading_h1(body),
    )))
}
