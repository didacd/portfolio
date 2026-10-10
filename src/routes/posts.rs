use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use dioxus_mdx::strip_leading_h1;

use crate::{content, render, state::AppState};

use super::internal_error;

pub(super) async fn show(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let Some((post, source)) = content::find_entry(&state.content_dir, &slug)
        .await
        .map_err(internal_error)?
    else {
        return Err(StatusCode::NOT_FOUND);
    };

    let (_, body) = content::parse_frontmatter(&source).map_err(internal_error)?;
    Ok(Html(render::post(&post, strip_leading_h1(body))))
}
