# Callouts

Use a blockquote that starts with a configured callout name:

```md
> [!note] This is a callout.
> This is regular text. You can also add **bold**, *italic*,
> [links](https://example.com), and `inline code`.
```

The first line supplies the callout's personalized title. The marker is removed,
and the following quoted lines become the normally rendered Markdown body. If
the first line has no title, the configured title is used instead.

## Custom callouts

Add a callout under `profile.callouts` in `src/config/site.yaml`:

```yaml
profile:
  callouts:
    release:
      icon: rocket
      title: Release
      color: "#22c55e"
```

`icon` must be a [Lucide icon](https://lucide.dev/icons/). `color` controls
the header, border, background tint, and body-text tint. Then use it in MDX:

```md
> [!release] Version 1.0 is now available.
```
