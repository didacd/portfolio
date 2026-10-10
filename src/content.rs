mod metadata;
pub(crate) mod profile;

use std::{collections::HashSet, io, path::Path};

use tokio::fs;

pub(crate) use metadata::{Metadata, parse as parse_frontmatter, safe_link};

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct EntrySummary {
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) file_name: String,
    pub(crate) metadata: Metadata,
    pub(crate) reading_minutes: usize,
}

impl EntrySummary {
    pub(crate) fn date(&self) -> Option<&str> {
        self.metadata
            .created_at
            .as_deref()
            .or(self.metadata.date.as_deref())
    }
}

pub(crate) async fn list_entries(content_dir: &Path) -> io::Result<Vec<EntrySummary>> {
    let mut directories = vec![content_dir.to_path_buf()];
    let mut posts = Vec::new();
    let mut slugs = HashSet::new();

    while let Some(directory) = directories.pop() {
        let mut entries = fs::read_dir(directory).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            let path = entry.path();
            if file_type.is_dir() {
                if !entry.file_name().to_string_lossy().starts_with('.') {
                    directories.push(path);
                }
                continue;
            }
            // Never follow symlinks out of the publishing directory.
            if !file_type.is_file()
                || entry.file_name().to_string_lossy().starts_with('.')
                || !is_markdown_file(&path)
            {
                continue;
            }

            let source = fs::read_to_string(&path).await?;
            if let Some(mut post) = summarize(&path, &source)? {
                if !slugs.insert(post.slug.clone()) {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("duplicate content slug: {}", post.slug),
                    ));
                }
                post.file_name = path
                    .strip_prefix(content_dir)
                    .map_err(io::Error::other)?
                    .to_string_lossy()
                    .into_owned();
                posts.push(post);
            }
        }
    }

    posts.sort_by(|left, right| {
        right
            .date()
            .cmp(&left.date())
            .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
    });
    Ok(posts)
}

pub(crate) async fn find_entry(
    content_dir: &Path,
    slug: &str,
) -> io::Result<Option<(EntrySummary, String)>> {
    if slug.is_empty()
        || !slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Ok(None);
    }

    if let Some(entry) = list_entries(content_dir)
        .await?
        .into_iter()
        .find(|entry| entry.slug == slug)
    {
        let path = content_dir.join(&entry.file_name);
        let source = fs::read_to_string(&path).await?;
        // Re-check publication flags in case a sync changed the file after discovery.
        if let Some(mut current) = summarize(&path, &source)? {
            current.file_name = entry.file_name;
            return Ok(Some((current, source)));
        }
    }

    Ok(None)
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("md") || extension.eq_ignore_ascii_case("mdx")
        })
}

fn summarize(path: &Path, source: &str) -> io::Result<Option<EntrySummary>> {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Ok(None);
    };
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let slug = slugify(stem);
    if slug.is_empty() {
        return Ok(None);
    }

    let (metadata, body) = parse_frontmatter(source)?;
    if metadata.draft || metadata.publish == Some(false) {
        return Ok(None);
    }
    let title = if metadata.title.is_empty() {
        first_heading(body).unwrap_or_else(|| title_from_slug(&slug))
    } else {
        metadata.title.clone()
    };

    Ok(Some(EntrySummary {
        slug,
        title,
        file_name: file_name.to_owned(),
        metadata,
        reading_minutes: body.split_whitespace().count().div_ceil(200).max(1),
    }))
}

fn first_heading(markdown: &str) -> Option<String> {
    markdown.lines().find_map(|line| {
        let heading = line.trim_start().strip_prefix("# ")?.trim();
        (!heading.is_empty()).then(|| heading.to_owned())
    })
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut separator = false;

    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            separator = false;
        } else if !slug.is_empty() && !separator {
            slug.push('-');
            separator = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

fn title_from_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{find_entry, first_heading, list_entries, slugify, summarize, title_from_slug};
    use std::{fs, path::Path};

    #[test]
    fn slugs_are_url_safe_and_title_fallback_is_readable() {
        let slug = slugify("My Rust Portfolio!");
        assert_eq!(slug, "my-rust-portfolio");
        assert_eq!(title_from_slug(&slug), "My Rust Portfolio");
    }

    #[test]
    fn reads_the_first_level_one_heading() {
        assert_eq!(
            first_heading("\n# A title\n\n## A subsection"),
            Some("A title".to_owned())
        );
        assert_eq!(first_heading("## Not a page title"), None);
    }

    #[test]
    fn title_fallbacks_and_publication_flags_work_for_both_extensions() {
        let path = Path::new("a-file.mdx");
        let entry = summarize(path, "---\ntitle: Explicit title\n---\n# Heading")
            .unwrap()
            .unwrap();
        assert_eq!(entry.title, "Explicit title");
        assert_eq!(
            summarize(path, "# Heading").unwrap().unwrap().title,
            "Heading"
        );
        assert_eq!(summarize(path, "Body").unwrap().unwrap().title, "A File");
        for name in ["post.md", "post.mdx"] {
            for flag in ["draft: true", "publish: false"] {
                assert!(
                    summarize(Path::new(name), &format!("---\n{flag}\n---\nPrivate"))
                        .unwrap()
                        .is_none()
                );
            }
        }
    }

    #[tokio::test]
    async fn discovers_nested_entries_orders_by_date_and_hides_private_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("nested")).unwrap();
        fs::create_dir(dir.path().join(".obsidian")).unwrap();
        for (name, source) in [
            (
                "older.md",
                "---\ntitle: Z older\ncreatedAt: 08-28-2026\n---\nOld",
            ),
            (
                "nested/newer.mdx",
                "---\ntitle: A newer\ncreatedAt: 2026-09-01\n---\nNew",
            ),
            ("undated.md", "# Undated"),
            ("draft.md", "---\ndraft: true\n---\nPrivate"),
            ("private.mdx", "---\npublish: false\n---\nPrivate"),
            (".hidden.md", "Hidden"),
            (".obsidian/secret.md", "Hidden"),
            ("attachment.txt", "Not Markdown"),
        ] {
            fs::write(dir.path().join(name), source).unwrap();
        }
        let entries = list_entries(dir.path()).await.unwrap();
        assert_eq!(
            entries
                .iter()
                .map(|entry| entry.slug.as_str())
                .collect::<Vec<_>>(),
            ["newer", "older", "undated"]
        );
        let (entry, source) = find_entry(dir.path(), "newer").await.unwrap().unwrap();
        assert_eq!(entry.file_name, "nested/newer.mdx");
        assert!(source.contains("A newer"));
        for slug in ["draft", "private", "../older", "", "OLDER"] {
            assert!(find_entry(dir.path(), slug).await.unwrap().is_none());
        }
    }

    #[tokio::test]
    async fn duplicate_slugs_are_reported_instead_of_picking_an_arbitrary_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("Same name.md"), "First").unwrap();
        fs::write(dir.path().join("same-name.mdx"), "Second").unwrap();
        assert!(
            list_entries(dir.path())
                .await
                .unwrap_err()
                .to_string()
                .contains("duplicate content slug")
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn symlinks_do_not_publish_files_outside_the_content_directory() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("private.md"), "Private").unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("external")).unwrap();
        std::os::unix::fs::symlink(
            outside.path().join("private.md"),
            dir.path().join("post.md"),
        )
        .unwrap();
        assert!(list_entries(dir.path()).await.unwrap().is_empty());
    }
}
