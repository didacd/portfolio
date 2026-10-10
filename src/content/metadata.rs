use std::io;

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(default)]
pub(crate) struct Metadata {
    pub(crate) title: String,
    pub(crate) description: String,
    #[serde(rename = "createdAt")]
    pub(crate) created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub(crate) updated_at: Option<String>,
    pub(crate) date: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) image: Option<String>,
    pub(crate) draft: bool,
    pub(crate) publish: Option<bool>,
    pub(crate) link: Option<String>,
    pub(crate) info: Vec<ProjectInfo>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub(crate) struct ProjectInfo {
    pub(crate) text: String,
    pub(crate) link: Option<String>,
    // Icon metadata from v1 is retained; unsupported names needn't affect the text.
    pub(crate) icon: Option<Icon>,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub(crate) struct Icon {
    #[serde(rename = "type")]
    pub(crate) kind: String,
    pub(crate) name: String,
}

pub(crate) fn parse(source: &str) -> io::Result<(Metadata, &str)> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let Some(first_newline) = source.find('\n') else {
        return Ok((Metadata::default(), source));
    };
    if source[..first_newline].trim_end() != "---" {
        return Ok((Metadata::default(), source));
    }

    let mut offset = first_newline + 1;
    for line in source[offset..].split_inclusive('\n') {
        if matches!(line.trim_end(), "---" | "...") {
            let yaml = &source[first_newline + 1..offset];
            let mut metadata: Metadata = serde_saphyr::from_str(yaml).map_err(io::Error::other)?;
            for value in [
                &mut metadata.created_at,
                &mut metadata.updated_at,
                &mut metadata.date,
            ]
            .into_iter()
            .flatten()
            {
                *value = normalize_date(value)?;
            }
            metadata.tags.sort();
            metadata.tags.dedup();
            return Ok((metadata, &source[offset + line.len()..]));
        }
        offset += line.len();
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "unterminated frontmatter",
    ))
}

fn normalize_date(value: &str) -> io::Result<String> {
    let invalid = || io::Error::new(io::ErrorKind::InvalidData, format!("invalid date: {value}"));
    let date = value.split('T').next().unwrap_or(value);
    let parts: Vec<_> = date.split('-').collect();
    if parts.len() != 3 {
        return Err(invalid());
    }
    let (year, month, day) = if parts[0].len() == 4 {
        (parts[0], parts[1], parts[2])
    } else if parts[2].len() == 4 {
        (parts[2], parts[0], parts[1])
    } else {
        return Err(invalid());
    };
    let year: u32 = year.parse().map_err(|_| invalid())?;
    let month: u32 = month.parse().map_err(|_| invalid())?;
    let day: u32 = day.parse().map_err(|_| invalid())?;
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        _ => return Err(invalid()),
    };
    if year == 0 || year > 9999 || day == 0 || day > days {
        return Err(invalid());
    }
    Ok(format!("{year:04}-{month:02}-{day:02}"))
}

pub(crate) fn safe_link(value: &str) -> bool {
    !value.chars().any(char::is_control)
        && (value.starts_with("https://")
            || value.starts_with("http://")
            || value.starts_with("mailto:")
            || (value.starts_with('/') && !value.starts_with("//") && !value.contains('\\')))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_v1_metadata_and_obsidian_properties() {
        let (meta, body) = parse("---\ntitle: Title\ncreatedAt: 08-28-2026\ndraft: true\ntags: [agents, personal]\ncustom: ignored\n---\nBody").unwrap();
        assert_eq!(meta.created_at.as_deref(), Some("2026-08-28"));
        assert_eq!(meta.tags, ["agents", "personal"]);
        assert!(meta.draft);
        assert_eq!(body, "Body");
    }

    #[test]
    fn reads_nested_project_info() {
        let (meta, _) = parse("---\ntitle: Project\ndate: 2026-08-28\ninfo:\n  - text: Source\n    link: https://github.com/didacd/portfolio\n    icon:\n      type: lucide\n      name: github\n---\nContent").unwrap();
        assert_eq!(meta.info[0].text, "Source");
        assert_eq!(meta.info[0].icon.as_ref().unwrap().name, "github");
    }

    #[test]
    fn rejects_invalid_dates_and_frontmatter_instead_of_publishing_it() {
        assert!(normalize_date("2026-02-29").is_err());
        assert_eq!(normalize_date("2024-02-29").unwrap(), "2024-02-29");
        assert!(parse("---\ndraft: invalid\n---\nBody").is_err());
        assert!(parse("---\ntitle: Missing delimiter").is_err());
        assert!(!safe_link("javascript:alert(1)"));
        assert!(!safe_link("//evil.example"));
    }
}
