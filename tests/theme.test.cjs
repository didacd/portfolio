const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { test } = require("node:test");
const { runInNewContext } = require("node:vm");

const script = readFileSync(new URL("../assets/theme.js", `file://${__filename}`), "utf8");

function setup({ saved = null, dark = false, blocked = false } = {}) {
  const events = {};
  const root = { dataset: {} };
  const attributes = {};
  const button = {
    hidden: true,
    setAttribute: (name, value) => { attributes[name] = value; },
    addEventListener: (name, callback) => { events[name] = callback; },
  };
  const media = {
    matches: dark,
    addEventListener: (name, callback) => { events[`media:${name}`] = callback; },
  };
  const storage = new Map(saved === null ? [] : [["portfolio-theme", saved]]);
  const context = {
    document: {
      documentElement: root,
      getElementById: () => button,
      addEventListener: (name, callback) => { events[name] = callback; },
    },
    window: {
      matchMedia: () => media,
      localStorage: {
        getItem: (key) => {
          if (blocked) throw new Error("Storage blocked");
          return storage.get(key) ?? null;
        },
        setItem: (key, value) => {
          if (blocked) throw new Error("Storage blocked");
          storage.set(key, value);
        },
      },
      addEventListener: (name, callback) => { events[name] = callback; },
    },
  };
  runInNewContext(script, context);
  return { root, attributes, button, media, storage, events };
}

test("saved preference is applied before DOMContentLoaded", () => {
  for (const saved of ["light", "dark"]) {
    const state = setup({ saved, dark: saved === "light" });
    assert.equal(state.root.dataset.theme, saved);
    assert.equal(state.button.hidden, true);
    state.events.DOMContentLoaded();
    assert.equal(state.button.hidden, false);
    assert.equal(state.attributes["aria-checked"], String(saved === "dark"));
  }
});

test("first visit follows the system theme and responds to system changes", () => {
  const state = setup();
  state.events.DOMContentLoaded();
  assert.equal(state.root.dataset.theme, undefined);
  assert.equal(state.attributes["aria-checked"], "false");
  state.media.matches = true;
  state.events["media:change"]();
  assert.equal(state.attributes["aria-checked"], "true");
});

test("switch toggles both modes, persists the choice, and overrides the system", () => {
  const state = setup({ dark: true });
  state.events.DOMContentLoaded();
  state.events.click();
  assert.equal(state.root.dataset.theme, "light");
  assert.equal(state.attributes["aria-checked"], "false");
  assert.equal(state.storage.get("portfolio-theme"), "light");
  state.events["media:change"]();
  assert.equal(state.attributes["aria-checked"], "false");
  state.events.click();
  assert.equal(state.root.dataset.theme, "dark");
  assert.equal(state.storage.get("portfolio-theme"), "dark");
});

test("blocked storage does not disable the switch", () => {
  const state = setup({ blocked: true });
  state.events.DOMContentLoaded();
  state.events.click();
  assert.equal(state.root.dataset.theme, "dark");
  assert.equal(state.attributes["aria-checked"], "true");
});

test("invalid stored preferences are ignored", () => {
  const state = setup({ saved: "invalid", dark: true });
  state.events.DOMContentLoaded();
  assert.equal(state.root.dataset.theme, undefined);
  assert.equal(state.attributes["aria-checked"], "true");
});

test("changes from other tabs update the theme; clearing restores system mode", () => {
  const state = setup({ saved: "dark" });
  state.events.DOMContentLoaded();
  state.events.storage({ key: "portfolio-theme", newValue: "light" });
  assert.equal(state.root.dataset.theme, "light");
  assert.equal(state.attributes["aria-checked"], "false");
  state.events.storage({ key: "unrelated", newValue: "dark" });
  assert.equal(state.root.dataset.theme, "light");
  state.events.storage({ key: "portfolio-theme", newValue: null });
  assert.equal(state.root.dataset.theme, undefined);
});
