# Arcanum — Rust Portfolio

A small server-rendered portfolio with a Rust/Axum server, Dioxus HTML rendering, and `dioxus-mdx` for Markdown and static MDX components. Locally bundled PrismJS and KaTeX provide browser-side code and math rendering.

## Run locally

```sh
cargo run
```

Open <http://127.0.0.1:3000>. Set `PORTFOLIO_ADDR` to change the listen address, `CONTENT_DIR` to use a different posts folder, or `PROJECTS_DIR` for a different projects folder.

The homepage (`/`) includes your biography from `content/about.md`, profile/social links and work history from `content/profile.json`, the two latest posts, and the two latest projects. The biography, profile image, social links, work history, and two published blog posts have been migrated from `master`; `notes.md` remains as the Markdown example.

Blog posts are listed at `/blog` and rendered at `/blog/{slug}`. Existing `/posts/{slug}` links still work. `/projects` lists directory-backed projects with detail pages at `/projects/{slug}`. There were no project entries on `master`, so the projects directory starts empty.

The homepage also loads GitHub statistics for the configured username. Requests have short timeouts and a ten-minute in-memory cache, including failures; the local profile still renders if GitHub is unavailable. `GITHUB_TOKEN` is optional and used only by the server. Set `GITHUB_STATS=off` to disable those requests. As on `master`, stars/languages summarize up to the first 100 public repositories, excluding forks.

## Add content

Put `.md` and `.mdx` files in `content/posts/` or `content/projects/`. Subdirectories are discovered recursively, but URLs use only the file's URL-safe stem (for example, `cloud/My notes.md` becomes `/blog/my-notes`). Stems must be unique within each collection; collisions are reported rather than choosing a file arbitrarily. Hidden paths and symlinks are excluded.

A frontmatter `title` takes precedence; otherwise, the first level-one heading is used, then the file name. Both `.md` and `.mdx` support YAML frontmatter, including nested project info. Frontmatter is removed before rendering. Entries with `draft: true` or `publish: false` are excluded from listings and direct URLs. Invalid metadata returns an error rather than publishing a potentially private entry. Without either flag, every discovered Markdown/MDX file is published.

### Blog post

```yaml
---
title: My post
description: A short summary.
createdAt: 2026-10-10
updatedAt: 2026-10-11 # optional
tags: [cloud, security]
image: /assets/images/pfp.png # optional
draft: false
---
```

Listings are newest first, with undated entries last. Dates accept `YYYY-MM-DD`, ISO timestamps (displayed as their date), and `MM-DD-YYYY` from v1. Tags are derived from published posts; `/blog?tags=cloud,security` matches posts containing **all** selected tags. Articles show descriptions, dates, tags, estimated reading time, optional cover images, and a table of contents. On wide screens, the contents sit in a separate sticky left sidebar and follow scrolling; on smaller screens, they appear above the article. Contents use hyphens rather than bullet dots.

Entry dates display full English month names (for example, `28 August 2026`), while HTML `datetime` attributes retain ISO dates. Publication, update, and project dates have Lucide calendar icons; each tag has a Lucide tag icon in previews, article headers, and blog filters.

### Project

Create a file such as `content/projects/my-project.mdx`:

```markdown
---
title: My project
description: What it does.
date: 2026-10-10
image: /assets/images/my-project.png
link: https://example.com
info:
  - text: Source code
    link: https://github.com/username/repository
    icon:
      type: lucide
      name: github
  - text: Rust / Kubernetes
---

## Overview

Describe the project here.
```

Projects sort by date, newest first. Detail pages show the cover image, optional project link, information items (with optional links and supported Lucide icons), and Markdown/MDX body. `image`, `link`, and `info` are optional. Unknown icon names may fall back to a generic icon; text and links remain usable. Put public images in `assets/images/` and reference `/assets/images/...`; imported v1 image paths have been updated accordingly. Content directories themselves are not exposed as static files.

If using Easy Git with Obsidian, keep mapping only the publishing folder to `content/posts`, not the entire vault. Wikilinks/embeds and attachment paths still need export conversion; attachments must be published to a served assets path. The site does not read Obsidian Sync directly.

The Rust MDX renderer supports Markdown plus its built-in static components, such as `<Tip>`, `<Note>`, `<Warning>`, cards, tabs, and steps. It does not execute JavaScript/React imports or expressions.

## Style the site

Edit `assets/site.css`. The stylesheet is served as a static asset, so CSS changes are visible on refresh without rebuilding the Rust application.

The profile image sits in a right-edge portrait card outside the navigation bar. Hover or keyboard focus reveals the complete image with a slight rotation; tapping toggles it, and Escape or tapping outside dismisses it. On narrow screens the card stays in the page flow to avoid covering content. Reduced-motion preferences disable its animation.

The header's Dark mode switch overrides the system theme and remembers the choice in browser local storage. With no saved preference (or JavaScript disabled), the site follows the system theme. Colors use CSS `light-dark()` so page, syntax, and callout colors all follow the selected mode.

The site uses the installed `Michelangelus` family when available. Microsoft does not permit hosting or converting its font files as web fonts, so it falls back to Georgia elsewhere; a web-font license from the typeface owner is needed for a consistent downloaded font. Code uses the locally served JetBrains Mono font, distributed under the SIL Open Font License in `assets/fonts/jetbrains-mono-OFL.txt`.

Code fences are highlighted in the browser by the locally bundled PrismJS 1.30.0 build (`assets/prism/`), with a blue-toned theme defined in `assets/site.css`. Inline and display math use locally bundled KaTeX 0.16.22 (`assets/math/`). GitHub-style Markdown alerts (`> [!note]`, `> [!tip]`, `> [!warning]`, and related types) are converted to the site's static callout components on the server. These enhancements do not execute MDX JavaScript; code remains readable as plain text if scripts are disabled.

## Project layout

- `src/routes/` defines the Axum route handlers.
- `src/render/` owns page and document rendering.
- Project HTTP handling lives in `src/routes/project_handlers.rs`; both list and detail rendering live in `src/render/projects.rs`.
- `src/content.rs` discovers and loads published entries for both collections.
- `src/content/` parses metadata and loads the local profile.
- `src/github.rs` handles cached GitHub statistics separately from rendering.
- `src/state.rs` holds application configuration shared with handlers.
- `assets/` contains the stylesheet, fonts, and other static files.

## Dependencies

Dependencies use current stable crate releases, with exact resolved versions recorded in `Cargo.lock`. After updating dependencies, run `cargo check` and `cargo test`; `cargo audit` can be used to check the lockfile against the RustSec advisory database.

## Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
node --test tests/*.test.cjs
```
