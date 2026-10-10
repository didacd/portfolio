(() => {
  const aliases = {
    bash: "bash",
    cjs: "javascript",
    css: "css",
    diff: "diff",
    htm: "markup",
    html: "markup",
    js: "javascript",
    jsx: "jsx",
    json: "json",
    md: "markdown",
    markdown: "markdown",
    py: "python",
    python: "python",
    rs: "rust",
    rust: "rust",
    sh: "bash",
    shell: "bash",
    toml: "toml",
    ts: "typescript",
    tsx: "tsx",
    typescript: "typescript",
    xml: "markup",
    yaml: "yaml",
    yml: "yaml",
  };

  function codeLabel(pre) {
    const header = pre.closest(".dk-code-block")?.firstElementChild;
    if (!header) return "";

    const tabs = [...header.querySelectorAll("button")];
    if (tabs.length > 1) {
      const activeTab = tabs.find((tab) => tab.className.includes("border-primary"));
      return (activeTab || tabs[0]).textContent.trim();
    }

    return header.querySelector("span")?.textContent.trim() || "";
  }

  function prismLanguage(pre, code) {
    const className = [...pre.classList, ...code.classList].find((name) =>
      name.startsWith("language-"),
    );
    let label = className ? className.slice("language-".length) : codeLabel(pre);
    label = label.trim().toLowerCase();

    const extension = label.match(/\.([a-z0-9]+)$/)?.[1];
    const language = aliases[extension || label.split(/\s+/)[0]];
    return language && Prism.languages[language] ? language : null;
  }

  document.querySelectorAll(".prose-content pre.dk-code").forEach((pre) => {
    const code = pre.querySelector("code");
    if (!code) return;

    const language = prismLanguage(pre, code);
    if (!language) return;

    code.textContent = code.textContent;
    code.className = `language-${language}`;
    pre.classList.add(`language-${language}`);
    Prism.highlightElement(code);
  });
})();
