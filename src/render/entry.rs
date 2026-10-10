use dioxus::prelude::*;

use crate::content::{EntrySummary, safe_link};

use super::icons::LucideIcon;

#[component]
pub(super) fn EntryPreview(entry: EntrySummary, section: String) -> Element {
    rsx! {
        li { class: "post-card",
            a { class: "post-link", href: "/{section}/{entry.slug}", "{entry.title}" }
            if let Some(date) = entry.date() {
                p { class: "post-meta",
                    DateLabel { date: date.to_owned() }
                }
            }
            if !entry.metadata.description.is_empty() {
                p { class: "entry-description", "{entry.metadata.description}" }
            }
            if !entry.metadata.tags.is_empty() {
                EntryTags { tags: entry.metadata.tags.clone() }
            }
        }
    }
}

#[component]
pub(super) fn EntryHeader(entry: EntrySummary, is_project: bool) -> Element {
    rsx! {
        header { class: "post-header", id: "top",
            h1 { "{entry.title}" }
            if !entry.metadata.description.is_empty() {
                p { class: "intro-copy", "{entry.metadata.description}" }
            }
            div { class: "article-meta",
                if let Some(date) = entry.date() {
                    DateLabel { date: date.to_owned() }
                }
                if let Some(updated) = &entry.metadata.updated_at {
                    DateLabel { date: updated.clone(), updated: true }
                }
                if !is_project {
                    span { "{entry.reading_minutes.max(1)} min read" }
                }
            }
            if !entry.metadata.tags.is_empty() {
                EntryTags { tags: entry.metadata.tags.clone() }
            }
            if let Some(image) = entry.metadata.image.as_deref().filter(|url| safe_link(url) && !url.starts_with("mailto:")) {
                img { class: "article-cover", src: image, alt: "", loading: "lazy" }
            }
        }
    }
}

#[component]
fn MetadataIcon(name: String) -> Element {
    rsx! {
        span { class: "metadata-icon", aria_hidden: "true", "data-icon": "{name}",
            LucideIcon { name: name.clone() }
        }
    }
}

#[component]
fn DateLabel(date: String, #[props(default)] updated: bool) -> Element {
    rsx! {
        span { class: "metadata-label date-label",
            MetadataIcon { name: "calendar".to_owned() }
            if updated { span { "Updated" } }
            time { datetime: "{date}", "{display_date(&date)}" }
        }
    }
}

#[component]
pub(super) fn TagLabel(tag: String) -> Element {
    rsx! {
        MetadataIcon { name: "tag".to_owned() }
        span { "{tag}" }
    }
}

#[component]
fn EntryTags(tags: Vec<String>) -> Element {
    rsx! {
        ul { class: "entry-tags", aria_label: "Tags", role: "list",
            for tag in &tags {
                li {
                    a { class: "metadata-label", href: filter_href(std::slice::from_ref(tag)),
                        TagLabel { tag: tag.clone() }
                    }
                }
            }
        }
    }
}

// Dates have already been validated and normalized by content loading.
// Keep presentation separate so sorting and HTML datetime values remain ISO dates.
fn display_date(date: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let formatted = (|| {
        let (year, rest) = date.split_once('-')?;
        let (month, day) = rest.split_once('-')?;
        let month = month.parse::<usize>().ok()?.checked_sub(1)?;
        let month = MONTHS.get(month)?;
        let day = day.parse::<u8>().ok()?;
        if !(1..=31).contains(&day) {
            return None;
        }
        Some(format!("{day} {month} {year}"))
    })();
    formatted.unwrap_or_else(|| date.to_owned())
}

pub(super) fn filter_href(tags: &[String]) -> String {
    if tags.is_empty() {
        return "/blog".to_owned();
    }
    let mut url = reqwest::Url::parse("https://example.invalid/blog").expect("constant URL");
    url.query_pairs_mut().append_pair("tags", &tags.join(","));
    format!("/blog?{}", url.query().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_every_month_with_unpadded_days() {
        for (month, name) in [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(
                display_date(&format!("2026-{:02}-01", month + 1)),
                format!("1 {name} 2026")
            );
        }
        assert_eq!(display_date("2026-08-28"), "28 August 2026");
        assert_eq!(display_date("2024-02-29"), "29 February 2024");
    }

    #[test]
    fn unexpected_date_values_do_not_panic() {
        for value in [
            "",
            "invalid",
            "2026-00-01",
            "2026-13-01",
            "2026-01-00",
            "2026-01-32",
        ] {
            assert_eq!(display_date(value), value);
        }
    }

    fn entry() -> EntrySummary {
        EntrySummary {
            title: "Example".to_owned(),
            metadata: crate::content::Metadata {
                created_at: Some("2026-08-28".to_owned()),
                updated_at: Some("2026-09-01".to_owned()),
                tags: vec!["cloud".to_owned(), "security".to_owned()],
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn previews_pair_each_tag_and_date_with_an_icon() {
        let html = dioxus_ssr::render_element(
            rsx! { EntryPreview { entry: entry(), section: "blog".to_owned() } },
        );
        assert!(html.contains("datetime=\"2026-08-28\">28 August 2026</time>"));
        assert_eq!(html.matches("data-icon=\"calendar\"").count(), 1);
        assert_eq!(html.matches("data-icon=\"tag\"").count(), 2);
        assert_eq!(html.matches("<svg").count(), 3);
        assert!(html.contains("<span>cloud</span>"));
        assert!(html.contains("<span>security</span>"));
    }

    #[test]
    fn article_dates_include_formatted_publication_and_update_dates() {
        let html =
            dioxus_ssr::render_element(rsx! { EntryHeader { entry: entry(), is_project: false } });
        assert!(html.contains("datetime=\"2026-08-28\">28 August 2026</time>"));
        assert!(html.contains("datetime=\"2026-09-01\">1 September 2026</time>"));
        assert!(html.contains("<span>Updated</span>"));
        assert_eq!(html.matches("data-icon=\"calendar\"").count(), 2);
        assert_eq!(html.matches("data-icon=\"tag\"").count(), 2);
    }
}
