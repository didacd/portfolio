use dioxus::prelude::*;

use crate::content::EntrySummary;

use super::{
    document_with_description, entry::EntryHeader, markdown::MarkdownContent, shell::SiteShell,
};

pub(crate) fn post(post: &EntrySummary, content: &str) -> String {
    let headings = dioxus_mdx::extract_headers(content);
    let body = dioxus_ssr::render_element(rsx! {
        SiteShell {
            main { class: if headings.is_empty() { "page-shell post-page" } else { "post-layout post-page" },
                div { class: "post-intro",
                    a { class: "back-link", href: "/blog", "← Back to blog" }
                    EntryHeader { entry: post.clone(), is_project: false }
                }
                if !headings.is_empty() {
                    aside { class: "toc-sidebar",
                        nav { class: "table-of-contents", aria_label: "Table of contents",
                            h2 { "On this page" }
                            ul { role: "list",
                                for (id, title, level) in &headings {
                                    li { "data-depth": "{level}",
                                        span { class: "toc-marker", aria_hidden: "true", "-" }
                                        a { href: "#{id}", "{title}" }
                                    }
                                }
                            }
                        }
                    }
                }
                article { class: "prose-content",
                    MarkdownContent { content: content.to_owned() }
                }
                a { class: "back-link article-back", href: "/blog", "← Back to blog" }
            }
        }
    });

    document_with_description(
        &format!("{} | Arcanum", post.title),
        &post.metadata.description,
        &body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrated_post_has_metadata_contents_links_and_matching_heading_anchors() {
        let source = include_str!("../../content/posts/skills-need-guardrails.mdx");
        let (metadata, content) = crate::content::parse_frontmatter(source).unwrap();
        let entry = EntrySummary {
            title: metadata.title.clone(),
            metadata,
            reading_minutes: 5,
            ..Default::default()
        };
        let html = post(&entry, content);
        assert!(html.contains("datetime=\"2026-08-28\""));
        assert!(html.contains("5 min read"));
        assert!(html.contains("href=\"/blog?tags=agents\""));
        assert!(html.contains("src=\"/assets/images/pfp.png\""));
        assert!(html.contains("href=\"#from-clever-prompts-to-dependable-procedures\""));
        assert!(html.contains("id=\"from-clever-prompts-to-dependable-procedures\""));
        assert!(html.contains("markdown-callout--note"));
        assert!(!html.contains("createdAt:"));
        let sidebar_start = html.find("<aside class=\"toc-sidebar\">").unwrap();
        let sidebar_end = sidebar_start + html[sidebar_start..].find("</aside>").unwrap();
        let article_start = html.find("<article class=\"prose-content\">").unwrap();
        assert!(
            sidebar_end < article_start,
            "contents belong outside the article"
        );
        assert_eq!(
            html.matches("class=\"toc-marker\" aria-hidden=\"true\">-</span>")
                .count(),
            dioxus_mdx::extract_headers(content).len()
        );
    }

    #[test]
    fn posts_without_headings_do_not_reserve_a_sidebar() {
        let html = post(&EntrySummary::default(), "A short post.");
        assert!(html.contains("class=\"page-shell post-page\""));
        assert!(!html.contains("toc-sidebar"));
        assert!(!html.contains("On this page"));
    }
}
