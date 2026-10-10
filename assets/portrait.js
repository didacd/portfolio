(() => {
  const card = document.getElementById("portrait-card");
  if (!card) return;

  let hovered = false;
  let keyboardFocused = false;
  let pinned = false;

  function update() {
    const expanded = hovered || keyboardFocused || pinned;
    card.setAttribute("aria-expanded", String(expanded));
    card.setAttribute("aria-label", expanded
      ? "Hide full profile picture"
      : "Show full profile picture");
  }

  function close() {
    hovered = keyboardFocused = pinned = false;
    update();
  }

  card.setAttribute("data-enhanced", "true");
  card.addEventListener("pointerenter", (event) => {
    if (event.pointerType === "touch") return;
    hovered = true;
    update();
  });
  card.addEventListener("pointerleave", () => {
    hovered = false;
    update();
  });
  card.addEventListener("focus", () => {
    keyboardFocused = card.matches(":focus-visible");
    update();
  });
  card.addEventListener("blur", () => {
    keyboardFocused = false;
    pinned = false;
    update();
  });
  card.addEventListener("click", () => {
    if (card.getAttribute("aria-expanded") === "true") {
      close();
    } else {
      pinned = true;
      update();
    }
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "Escape") close();
  });
  document.addEventListener("pointerdown", (event) => {
    if (!card.contains(event.target)) close();
  });
})();
