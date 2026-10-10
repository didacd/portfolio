use dioxus::prelude::*;

use crate::{
    content::{EntrySummary, profile::Profile, safe_link},
    github::GithubData,
};

use super::{
    document_with_description, entry::EntryPreview, markdown::MarkdownContent, shell::SiteShell,
};

pub(crate) fn home(
    content: &str,
    profile: &Profile,
    posts: &[EntrySummary],
    projects: &[EntrySummary],
    github: Option<&GithubData>,
) -> String {
    let body = dioxus_ssr::render_element(rsx! {
        SiteShell {
            brand_image: profile.avatar.clone(),
            main { class: "home-layout",
                aside { class: "profile-sidebar", aria_label: "Profile",
                    h2 { "{profile.name}" }
                    p { "{profile.role}" }
                    p { class: "profile-location", "{profile.location}" }
                    section { class: "profile-socials", aria_label: "Social links",
                        h3 { "Socials" }
                        ul {
                            for social in &profile.socials {
                                if safe_link(&social.link) {
                                    li { a { href: "{social.link}", "{social.text}" } }
                                }
                            }
                        }
                    }
                    if let Some(github) = github {
                        section { class: "github-summary", aria_label: "GitHub statistics",
                            h3 { "GitHub" }
                            p { "Member since {github.user.created_at.get(..4).unwrap_or_default()}" }
                            dl { class: "github-stats",
                                div { dt { "Repositories" } dd { "{github.user.public_repos}" } }
                                div { dt { "Followers" } dd { "{github.user.followers}" } }
                                div { dt { "Following" } dd { "{github.user.following}" } }
                                div { dt { "Stars" } dd { "{github.stars}" } }
                            }
                            if !github.languages.is_empty() {
                                h3 { "Top languages" }
                                ul { class: "profile-languages",
                                    for language in &github.languages { li { "{language}" } }
                                }
                            }
                        }
                    }
                }
                div { class: "home-content",
                    section { class: "home-section", aria_label: "About me",
                        h1 { "About me" }
                        div { class: "prose-content",
                            MarkdownContent { content: content.to_owned() }
                        }
                    }
                    section { class: "home-section", aria_label: "Latest posts",
                        div { class: "section-heading", h2 { "Latest posts" } a { href: "/blog", "All posts" } }
                        if posts.is_empty() { p { class: "empty-state", "No posts yet." } }
                        ul { class: "entry-previews",
                            for post in posts.iter().take(2) {
                                EntryPreview { entry: post.clone(), section: "blog".to_owned() }
                            }
                        }
                    }
                    section { class: "home-section", aria_label: "Latest projects",
                        div { class: "section-heading", h2 { "Latest projects" } a { href: "/projects", "All projects" } }
                        if projects.is_empty() { p { class: "empty-state", "No projects yet." } }
                        ul { class: "entry-previews",
                            for project in projects.iter().take(2) {
                                EntryPreview { entry: project.clone(), section: "projects".to_owned() }
                            }
                        }
                    }
                    section { class: "home-section", aria_label: "Work experience",
                        h2 { "Work experience" }
                        ol { class: "work-list",
                            for work in profile.work.iter().rev() {
                                li {
                                    p { class: "work-duration", "{work.duration}" }
                                    h3 { "{work.company}" }
                                    p { class: "work-role", "{work.title}" }
                                    p { "{work.description}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    document_with_description(
        "Didac | Arcanum",
        "Cloud security, infrastructure, and platform engineering. Didac's personal space.",
        &body,
    )
}

#[cfg(test)]
mod tests {
    use super::home;

    #[test]
    fn renders_profile_and_about_content_from_master() {
        let profile = serde_json::from_str(include_str!("../../content/profile.json")).unwrap();
        let html = home("My **introduction**.", &profile, &[], &[], None);
        assert!(html.contains("<h1>About me</h1>"));
        assert!(html.contains("My <strong>introduction</strong>."));
        assert!(html.contains("Cloud Security Engineer"));
        assert!(html.contains("Telefonica UK"));
        assert!(html.contains("https://mastodon.social/@didacdomenech"));
        assert_eq!(html.matches("src=\"/assets/images/pfp.png\"").count(), 1);
        assert!(!html.contains("class=\"profile-avatar\""));
    }
}
