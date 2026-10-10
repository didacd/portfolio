use dioxus::prelude::*;

use crate::content::safe_link;

#[component]
pub(super) fn SiteShell(
    children: Element,
    #[props(default = "/assets/images/pfp.png".to_owned())] brand_image: String,
) -> Element {
    rsx! {
        div { class: "site-frame",
            header { class: "site-header",
                a { class: "wordmark", href: "/",
                    span { "Arcanum" }
                }
                nav { aria_label: "Main navigation",
                    a { href: "/", "About" }
                    a { href: "/blog", "Blog" }
                    a { href: "/projects", "Projects" }
                    button {
                        class: "theme-switch",
                        id: "theme-switch",
                        r#type: "button",
                        role: "switch",
                        aria_label: "Dark mode",
                        aria_checked: "false",
                        hidden: true,
                        span { class: "theme-switch-track", aria_hidden: "true",
                            span { class: "theme-switch-thumb" }
                        }
                    }
                }
            }
            if safe_link(&brand_image) && !brand_image.starts_with("mailto:") {
                aside { class: "portrait-dock", aria_label: "Profile picture",
                    button {
                        class: "portrait-card",
                        id: "portrait-card",
                        r#type: "button",
                        aria_label: "Show full profile picture",
                        aria_expanded: "false",
                        img {
                            class: "portrait-image",
                            src: "{brand_image}",
                            width: "1254",
                            height: "1254",
                            alt: "Didac's illustrated portrait",
                            draggable: "false",
                        }
                    }
                }
            }
            {children}
            footer { class: "site-footer", "A portfolio of work and notes." }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_an_accessible_theme_switch() {
        let html = dioxus_ssr::render_element(rsx! {
            SiteShell { main { "Content" } }
        });
        assert!(html.contains("id=\"theme-switch\""));
        assert!(html.contains("role=\"switch\""));
        assert!(html.contains("aria-label=\"Dark mode\""));
        assert!(html.contains("aria-checked=\"false\""));
        assert!(html.contains("href=\"/blog\">Blog</a>"));
        assert!(html.contains("href=\"/projects\">Projects</a>"));
        assert!(!html.contains("Writing"));
    }

    #[test]
    fn places_an_accessible_portrait_card_outside_the_header() {
        let html = dioxus_ssr::render_element(rsx! {
            SiteShell { main { "Content" } }
        });
        let image = html.find("class=\"portrait-image\"").unwrap();
        let wordmark = html.find("<span>Arcanum</span>").unwrap();
        let header_end = html.find("</header>").unwrap();
        assert!(wordmark < header_end && header_end < image);
        assert!(html.contains("src=\"/assets/images/pfp.png\""));
        assert!(html.contains("aria-label=\"Show full profile picture\""));
        assert!(html.contains("aria-expanded=\"false\""));
        assert!(html.contains("id=\"portrait-card\" type=\"button\""));
    }
}
