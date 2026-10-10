use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
};
use serde::Deserialize;

use crate::{content, render, state::AppState};

use super::internal_error;

#[derive(Default, Deserialize)]
pub(super) struct Filter {
    tags: Option<String>,
}

pub(super) async fn index(
    State(state): State<AppState>,
    Query(filter): Query<Filter>,
) -> Result<Html<String>, StatusCode> {
    let posts = content::list_entries(&state.content_dir)
        .await
        .map_err(internal_error)?;

    let selected: Vec<_> = filter
        .tags
        .as_deref()
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(str::to_owned)
        .collect();
    Ok(Html(render::blog(&posts, &selected)))
}
