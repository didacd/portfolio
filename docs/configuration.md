# Configuration

Personalise the template through `src/config/site.yaml`. The homepage reads this
file at build time, so no social links or GitHub account details are hardcoded
in the page.

## Profile

Set `profile.github.username` to the GitHub account for the optional profile
card. If GitHub cannot be reached during a build, the card is simply omitted.

## Social links

Add entries to `profile.socials` to control the social-links card. Each entry
needs a unique numeric `id`, a display `text`, a `link`, and an icon:

```yaml
- id: 1
  icon:
    type: lucide # or simple-icons
    name: github
  text: GitHub
  link: https://github.com/example
```

The links appear in the same order as the YAML list. Icon names come from
[Lucide](https://lucide.dev/icons/) or [Simple Icons](https://simpleicons.org/).

See the [callout guide](./callouts.md) to configure MDX callouts.
