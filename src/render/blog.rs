use std::collections::BTreeSet;

use dioxus::prelude::*;

use crate::content::EntrySummary;

use super::{
    document,
    entry::{EntryPreview, TagLabel, filter_href},
    shell::SiteShell,
};

pub(crate) fn blog(posts: &[EntrySummary], selected: &[String]) -> String {
    let tags: BTreeSet<_> = posts
        .iter()
        .flat_map(|post| post.metadata.tags.iter())
        .collect();
    let filtered: Vec<_> = posts
        .iter()
        .filter(|post| selected.iter().all(|tag| post.metadata.tags.contains(tag)))
        .collect();
    let body = dioxus_ssr::render_element(rsx! {
        SiteShell {
            main { class: "page-shell",
                section { class: "intro",
                    h1 { "Blog" }
                }
                if !tags.is_empty() || !selected.is_empty() {
                    nav { class: "tag-filter", aria_label: "Filter posts by tags",
                        a { href: "/blog", "Clear filters" }
                        for tag in &tags {
                            a {
                                href: {
                                    let mut next = selected.to_vec();
                                    if next.contains(tag) { next.retain(|item| item != *tag); }
                                    else { next.push((*tag).clone()); }
                                    filter_href(&next)
                                },
                                class: if selected.contains(tag) { "tag active" } else { "tag" },
                                aria_current: if selected.contains(tag) { "true" } else { "false" },
                                TagLabel { tag: (*tag).clone() }
                            }
                        }
                    }
                }
                section { class: "content-list", aria_label: "Blog posts",
                    if filtered.is_empty() {
                        if selected.is_empty() {
                        p { class: "empty-state", "No posts yet." }
                        } else {
                            p { class: "empty-state", "No posts match these tags." }
                        }
                    } else {
                        ul { class: "post-list",
                            for post in filtered {
                                EntryPreview { entry: post.clone(), section: "blog".to_owned() }
                            }
                        }
                    }
                }
            }
        }
    });

    document("Blog | Arcanum", &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_posts_on_the_blog_page() {
        let html = blog(
            &[EntrySummary {
                slug: "notes".to_owned(),
                title: "Notes".to_owned(),
                file_name: "notes.md".to_owned(),
                ..Default::default()
            }],
            &[],
        );
        assert!(html.contains("<h1>Blog</h1>"));
        assert!(html.contains("href=\"/blog/notes\""));
        assert!(html.contains("Notes</a>"));
    }

    #[test]
    fn renders_an_empty_blog() {
        assert!(blog(&[], &[]).contains("No posts yet."));
    }

    #[test]
    fn tag_filters_require_all_selected_tags() {
        let posts = vec![EntrySummary {
            slug: "agents".to_owned(),
            title: "Agents".to_owned(),
            metadata: crate::content::Metadata {
                tags: vec!["agents".to_owned()],
                ..Default::default()
            },
            ..Default::default()
        }];
        assert!(blog(&posts, &["agents".to_owned()]).contains("href=\"/blog/agents\""));
        let html = blog(&posts, &[]);
        assert_eq!(
            html.matches("data-icon=\"tag\"").count(),
            2,
            "both filter and entry tags have icons"
        );
        assert!(
            !blog(&posts, &["agents".to_owned(), "personal".to_owned()])
                .contains("href=\"/blog/agents\"")
        );
    }
}
