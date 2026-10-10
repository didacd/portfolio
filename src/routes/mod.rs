mod blog;
mod home;
mod posts;
mod project_handlers;
#[cfg(test)]
mod tests;

use std::io;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::services::ServeDir;
use tracing::error;

use crate::state::AppState;

pub(crate) fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home::index))
        .route("/blog", get(blog::index))
        .route("/blog/{slug}", get(posts::show))
        .route("/projects", get(project_handlers::index))
        .route("/projects/{slug}", get(project_handlers::show))
        .route("/posts/{slug}", get(posts::show))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}

fn internal_error(error: io::Error) -> StatusCode {
    error!(%error, "failed to read portfolio content");
    StatusCode::INTERNAL_SERVER_ERROR
}
