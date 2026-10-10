use std::{fs, sync::Arc};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use crate::{github::GithubCache, state::AppState};

fn fixture() -> (tempfile::TempDir, Router) {
    let root = tempfile::tempdir().unwrap();
    let posts = root.path().join("posts");
    let projects = root.path().join("projects");
    fs::create_dir(&posts).unwrap();
    fs::create_dir(&projects).unwrap();
    fs::write(
        posts.join("public.mdx"),
        "---\ntitle: Public post\ncreatedAt: 08-28-2026\ntags: [agents]\n---\nVisible **body**.",
    )
    .unwrap();
    fs::write(
        posts.join("draft.md"),
        "---\ntitle: Hidden draft\ndraft: true\n---\nPrivate body.",
    )
    .unwrap();
    fs::write(
        posts.join("private.mdx"),
        "---\ntitle: Private post\npublish: false\n---\nPrivate body.",
    )
    .unwrap();
    fs::write(projects.join("example.mdx"), "---\ntitle: Example project\ndescription: A test fixture, not portfolio content.\ndate: 2026-09-15\nimage: /assets/images/pfp.png\nlink: https://example.com\ninfo:\n  - text: Source\n    link: https://github.com/didacd/portfolio\n    icon:\n      type: lucide\n      name: github\n  - text: Rust\n---\n# Example project\n\nProject **details**.").unwrap();
    let state = AppState {
        content_dir: posts,
        projects_dir: projects,
        github: Arc::new(GithubCache::new().unwrap()),
    };
    (root, super::router(state))
}

async fn request(app: &Router, uri: &str) -> (StatusCode, String) {
    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

#[tokio::test]
async fn posts_keep_both_url_schemes_and_hide_drafts_from_every_route() {
    let (_root, app) = fixture();
    let (status, listing) = request(&app, "/blog").await;
    assert_eq!(status, StatusCode::OK);
    assert!(listing.contains("Public post"));
    assert!(!listing.contains("Hidden draft"));
    assert!(!listing.contains("Private post"));
    let (status, canonical) = request(&app, "/blog/public").await;
    assert_eq!(status, StatusCode::OK);
    assert!(canonical.contains("Visible <strong>body</strong>."));
    let (status, legacy) = request(&app, "/posts/public").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(canonical, legacy);
    for uri in [
        "/blog/draft",
        "/posts/draft",
        "/blog/private",
        "/posts/private",
        "/blog/missing",
        "/blog/%2e%2e",
    ] {
        assert_eq!(request(&app, uri).await.0, StatusCode::NOT_FOUND, "{uri}");
    }
    let (_, filtered) = request(&app, "/blog?tags=agents").await;
    assert!(filtered.contains("href=\"/blog/public\""));
    let (_, filtered) = request(&app, "/blog?tags=personal").await;
    assert!(filtered.contains("No posts match these tags."));
    assert!(!filtered.contains("href=\"/blog/public\""));
}

#[tokio::test]
async fn project_lists_and_detail_pages_render_nested_metadata() {
    let (_root, app) = fixture();
    let (status, listing) = request(&app, "/projects").await;
    assert_eq!(status, StatusCode::OK);
    assert!(listing.contains("href=\"/projects/example\""));
    assert!(listing.contains("datetime=\"2026-09-15\""));
    assert!(listing.contains("15 September 2026</time>"));
    let (status, detail) = request(&app, "/projects/example").await;
    assert_eq!(status, StatusCode::OK);
    assert!(detail.contains("Project <strong>details</strong>."));
    assert!(detail.contains("15 September 2026</time>"));
    assert!(detail.contains("data-icon=\"calendar\""));
    assert!(detail.contains("href=\"https://github.com/didacd/portfolio\""));
    assert!(detail.contains("Visit project"));
    assert!(detail.contains("src=\"/assets/images/pfp.png\""));
    assert!(!detail.contains("info:"));
    assert!(!detail.contains("min read"));
    assert_eq!(detail.matches("<h1>Example project</h1>").count(), 1);
    assert_eq!(
        request(&app, "/projects/missing").await.0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn invalid_metadata_is_not_published() {
    let (root, app) = fixture();
    fs::write(
        root.path().join("posts/broken.md"),
        "---\ndraft: not-a-boolean\n---\nPrivate content",
    )
    .unwrap();
    let (status, body) = request(&app, "/blog/broken").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!body.contains("Private content"));
}
