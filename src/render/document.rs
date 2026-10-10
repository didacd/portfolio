pub(crate) fn document(title: &str, body: &str) -> String {
    document_with_description(title, "", body)
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub(crate) fn document_with_description(title: &str, description: &str, body: &str) -> String {
    let title = escape(title);
    let description = escape(description);

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <meta name="color-scheme" content="light dark">
    <meta name="description" content="{description}">
    <script src="/assets/theme.js"></script>
    <link rel="stylesheet" href="/assets/site.css">
    <link rel="stylesheet" href="/assets/math/katex.min.css">
    <title>{title}</title>
  </head>
  <body>
    {body}
    <script src="/assets/portrait.js" defer></script>
    <script src="/assets/prism/prism.js" defer></script>
    <script src="/assets/prism/highlight.js" defer></script>
    <script src="/assets/math/katex.min.js" defer></script>
    <script src="/assets/math/auto-render.min.js" defer></script>
    <script src="/assets/math/render.js" defer></script>
  </body>
</html>"#
    )
}

#[cfg(test)]
mod tests {
    use super::{document, document_with_description};

    #[test]
    fn escapes_the_document_title() {
        let page = document("A <title> & more", "<main>content</main>");
        assert!(page.contains("A &lt;title&gt; &amp; more"));
        assert!(page.contains("<main>content</main>"));
        assert!(page.contains("/assets/prism/prism.js"));
        assert!(page.contains("/assets/portrait.js"));
        assert!(page.contains("/assets/prism/highlight.js"));
        assert!(page.contains("/assets/math/katex.min.css"));
        assert!(page.contains("/assets/math/render.js"));
        assert!(page.find("/assets/theme.js").unwrap() < page.find("/assets/site.css").unwrap());
    }

    #[test]
    fn escapes_the_meta_description() {
        let page = document_with_description("Title", "A \"description\" <with> & markup", "");
        assert!(page.contains("content=\"A &quot;description&quot; &lt;with&gt; &amp; markup\""));
    }
}
