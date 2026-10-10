use dioxus::prelude::*;
use dioxus_mdx::{CalloutType, DocNode, DocNodeRenderer};

use super::icons::LucideIcon;

#[component]
pub(super) fn MarkdownContent(content: String) -> Element {
    let content = normalize_github_alerts(&content);
    let (nodes, _) = dioxus_mdx::parse_body(&content);
    rsx! {
        for (index, node) in nodes.into_iter().enumerate() {
            PostNode { key: "{index}", node }
        }
    }
}

#[component]
fn PostNode(node: DocNode) -> Element {
    match node {
        DocNode::Callout(callout) => {
            let (kind, title, content_html) =
                callout_presentation(callout.callout_type, &callout.content_html);
            rsx! {
                SiteCallout {
                    kind: kind.to_owned(),
                    title: title.to_owned(),
                    content_html,
                }
            }
        }
        node => rsx! { DocNodeRenderer { node } },
    }
}

#[component]
fn SiteCallout(kind: String, title: String, content_html: String) -> Element {
    let icon = match kind.as_str() {
        "note" => "pencil",
        "tip" => "flame",
        "warning" | "caution" => "triangle-alert",
        "error" => "circle-x",
        "faq" => "circle-help",
        "quote" => "quote",
        _ => "info",
    };

    rsx! {
        aside {
            class: "markdown-callout markdown-callout--{kind}",
            role: "alert",
            div { class: "markdown-callout-heading",
                span {
                    class: "markdown-callout-icon",
                    aria_hidden: "true",
                    "data-icon": icon,
                    LucideIcon { name: icon.to_owned() }
                }
                span { class: "markdown-callout-title", "{title}" }
            }
            div {
                class: "markdown-callout-content",
                dangerous_inner_html: content_html,
            }
        }
    }
}

fn callout_presentation(
    callout_type: CalloutType,
    html: &str,
) -> (&'static str, &'static str, String) {
    for (marker, kind, title) in [
        ("PORTFOLIOCALLOUTERRORMARKER", "error", "Error"),
        ("PORTFOLIOCALLOUTCAUTIONMARKER", "caution", "Caution"),
        ("PORTFOLIOCALLOUTIMPORTANTMARKER", "important", "Important"),
        ("PORTFOLIOCALLOUTFAQMARKER", "faq", "FAQ"),
        ("PORTFOLIOCALLOUTQUOTEMARKER", "quote", "Quote"),
    ] {
        if html.contains(marker) {
            let paragraph = format!("<p>{marker}</p>");
            return (kind, title, html.replace(&paragraph, "").trim().to_owned());
        }
    }

    let (kind, title) = match callout_type {
        CalloutType::Note => ("note", "Note"),
        CalloutType::Tip => ("tip", "Tip"),
        CalloutType::Warning => ("warning", "Warning"),
        CalloutType::Info => ("info", "Info"),
        _ => ("info", "Info"),
    };

    (kind, title, html.to_owned())
}

fn normalize_github_alerts(markdown: &str) -> String {
    let mut output = String::with_capacity(markdown.len());
    let mut lines = markdown.lines().peekable();
    let mut fence: Option<(char, usize)> = None;

    while let Some(line) = lines.next() {
        if let Some((fence_char, fence_len)) = fence {
            output.push_str(line);
            output.push('\n');
            if closes_fence(line, fence_char, fence_len) {
                fence = None;
            }
            continue;
        }

        if let Some((fence_char, fence_len)) = opening_fence(line) {
            fence = Some((fence_char, fence_len));
            output.push_str(line);
            output.push('\n');
            continue;
        }

        let Some((tag, marker, title)) = parse_github_alert(line) else {
            output.push_str(line);
            output.push('\n');
            continue;
        };

        let mut body = String::new();
        if let Some(marker) = marker {
            body.push_str(marker);
            body.push_str("\n\n");
        }
        body.push_str(title);

        while let Some(quoted_line) = lines.peek().copied() {
            let Some(quoted_content) = quoted_line.strip_prefix('>') else {
                break;
            };
            lines.next();
            if !body.is_empty() {
                body.push('\n');
            }
            body.push_str(quoted_content.strip_prefix(' ').unwrap_or(quoted_content));
        }

        output.push('<');
        output.push_str(tag);
        output.push_str(">\n");
        output.push_str(body.trim());
        output.push_str("\n</");
        output.push_str(tag);
        output.push_str(">\n\n");
    }

    output
}

fn parse_github_alert(line: &str) -> Option<(&'static str, Option<&'static str>, &str)> {
    let body = line.trim_start().strip_prefix('>')?.trim_start();
    let body = body.strip_prefix("[!")?;
    let close = body.find(']')?;
    let kind = body[..close].to_ascii_lowercase();
    let title = body[close + 1..].trim_start();

    let (tag, marker) = match kind.as_str() {
        "note" => ("Note", None),
        "info" => ("Info", None),
        "tip" => ("Tip", None),
        "warning" | "warn" => ("Warning", None),
        "caution" => ("Warning", Some("PORTFOLIOCALLOUTCAUTIONMARKER")),
        "error" => ("Warning", Some("PORTFOLIOCALLOUTERRORMARKER")),
        "important" => ("Info", Some("PORTFOLIOCALLOUTIMPORTANTMARKER")),
        "faq" => ("Info", Some("PORTFOLIOCALLOUTFAQMARKER")),
        "quote" => ("Info", Some("PORTFOLIOCALLOUTQUOTEMARKER")),
        _ => return None,
    };

    Some((tag, marker, title))
}

fn opening_fence(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start();
    let fence_char = trimmed.chars().next()?;
    if fence_char != '`' && fence_char != '~' {
        return None;
    }

    let fence_len = trimmed
        .chars()
        .take_while(|character| *character == fence_char)
        .count();
    (fence_len >= 3).then_some((fence_char, fence_len))
}

fn closes_fence(line: &str, fence_char: char, fence_len: usize) -> bool {
    let trimmed = line.trim_start();
    let length = trimmed
        .chars()
        .take_while(|character| *character == fence_char)
        .count();
    length >= fence_len && trimmed[length..].trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::{SiteCallout, normalize_github_alerts};
    use crate::content::EntrySummary;
    use crate::render::post as render_post;
    use dioxus::prelude::*;

    #[test]
    fn callouts_use_svg_icons_in_the_heading() {
        for (kind, icon_name) in [
            ("note", "pencil"),
            ("tip", "flame"),
            ("warning", "triangle-alert"),
            ("error", "circle-x"),
            ("info", "info"),
            ("faq", "circle-help"),
            ("quote", "quote"),
        ] {
            let rendered = dioxus_ssr::render_element(rsx! {
                SiteCallout {
                    kind: kind.to_owned(),
                    title: kind.to_owned(),
                    content_html: "<p>Content</p>".to_owned(),
                }
            });
            assert!(rendered.contains("<svg"), "{kind} needs an SVG icon");
            assert!(rendered.contains(&format!("data-icon=\"{icon_name}\"")));
            assert!(rendered.contains("aria-hidden=\"true\""));
            assert!(rendered.contains("<p>Content</p>"));
        }
    }

    #[test]
    fn renders_markdown_and_static_mdx_components() {
        let source = "A **Markdown** paragraph.\n\n<Tip>\nA static MDX callout.\n</Tip>";
        let summary = EntrySummary {
            slug: "example".to_owned(),
            title: "Example".to_owned(),
            file_name: "example.mdx".to_owned(),
            ..Default::default()
        };
        let rendered = render_post(&summary, source);

        assert!(rendered.contains("A <strong>Markdown</strong> paragraph."));
        assert!(rendered.contains("role=\"alert\""));
        assert!(rendered.contains("A static MDX callout."));
    }

    #[test]
    fn renders_the_full_markdown_example() {
        let source = include_str!("../../content/posts/notes.md");
        let body = dioxus_mdx::strip_leading_h1(dioxus_mdx::extract_frontmatter(source).1);
        let summary = EntrySummary {
            slug: "notes".to_owned(),
            title: "Notes in Markdown".to_owned(),
            file_name: "notes.md".to_owned(),
            ..Default::default()
        };
        let rendered = render_post(&summary, body);

        assert_eq!(rendered.matches("role=\"alert\"").count(), 6);
        for kind in ["note", "tip", "warning", "error", "faq", "quote"] {
            assert!(
                rendered.contains(&format!("markdown-callout--{kind}")),
                "missing {kind} callout"
            );
        }
        assert!(rendered.contains("markdown-callout-heading"));
        assert!(rendered.contains("markdown-callout-title\">Error</span>"));
        assert!(!rendered.contains("PORTFOLIOCALLOUT"));
        assert!(rendered.contains("<blockquote>"));
        assert!(rendered.contains("<h1>H1</h1>"));
        for level in 2..=6 {
            assert!(rendered.contains(&format!("<h{level}")), "missing h{level}");
        }
        assert!(rendered.contains("def main:"));
        assert!(rendered.contains("This is python code"));
        assert!(rendered.contains("$\\sqrt{3x-1}+(1+x)^2$"));
        assert!(rendered.contains("alt=\"cat-photo\""));
        assert!(rendered.contains("<hr"));
    }

    #[test]
    fn converts_github_alerts_without_rewriting_fenced_examples() {
        let markdown =
            "> [!quote] A quote\n> Continued.\n\n```md\n> [!note] Keep this literal\n```";
        let converted = normalize_github_alerts(markdown);

        assert!(
            converted
                .contains("<Info>\nPORTFOLIOCALLOUTQUOTEMARKER\n\nA quote\nContinued.\n</Info>")
        );
        assert!(converted.contains("> [!note] Keep this literal"));
        assert_eq!(converted.matches("<Info>").count(), 1);
    }
}
