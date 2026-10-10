use dioxus::prelude::*;
use dioxus_mdx::MdxIcon;

use crate::content::{EntrySummary, safe_link};

use super::{
    document, document_with_description,
    entry::{EntryHeader, EntryPreview},
    markdown::MarkdownContent,
    shell::SiteShell,
};

pub(crate) fn project_list(projects: &[EntrySummary]) -> String {
    let body = dioxus_ssr::render_element(rsx! {
        SiteShell {
            main { class: "page-shell",
                section { class: "intro",
                    h1 { "Projects" }
                    p { class: "intro-copy", "{projects.len()} projects" }
                }
                section { class: "content-list", aria_label: "Projects",
                    if projects.is_empty() {
                        p { class: "empty-state", "No projects yet." }
                    } else {
                        ul { class: "post-list",
                            for project in projects {
                                EntryPreview { entry: project.clone(), section: "projects".to_owned() }
                            }
                        }
                    }
                }
            }
        }
    });

    document("Projects | Arcanum", &body)
}

pub(crate) fn project_detail(project: &EntrySummary, content: &str) -> String {
    let body = dioxus_ssr::render_element(rsx! {
        SiteShell {
            main { class: "page-shell post-page",
                a { class: "back-link", href: "/projects", "← Back to projects" }
                EntryHeader { entry: project.clone(), is_project: true }
                if !project.metadata.info.is_empty() || project.metadata.link.is_some() {
                    section { class: "project-info", aria_label: "Project information",
                        h2 { "Project information" }
                        ul {
                            if let Some(link) = project.metadata.link.as_deref().filter(|link| safe_link(link)) {
                                li { a { href: link, "Visit project" } }
                            }
                            for info in &project.metadata.info {
                                li {
                                    if let Some(icon) = &info.icon {
                                        if icon.kind == "lucide" {
                                            span { class: "project-info-icon", aria_hidden: "true",
                                                MdxIcon { name: icon.name.clone() }
                                            }
                                        }
                                    }
                                    if let Some(link) = info.link.as_deref().filter(|link| safe_link(link)) {
                                        a { href: link, "{info.text}" }
                                    } else {
                                        span { "{info.text}" }
                                    }
                                }
                            }
                        }
                    }
                }
                article { class: "prose-content",
                    MarkdownContent { content: content.to_owned() }
                }
                a { class: "back-link article-back", href: "/projects", "← Back to projects" }
            }
        }
    });

    document_with_description(
        &format!("{} | Arcanum", project.title),
        &project.metadata.description,
        &body,
    )
}

#[cfg(test)]
mod tests {
    use super::project_list;

    #[test]
    fn renders_a_projects_page_without_fabricated_projects() {
        let html = project_list(&[]);
        assert!(html.contains("<h1>Projects</h1>"));
        assert!(html.contains("No projects yet."));
    }
}
