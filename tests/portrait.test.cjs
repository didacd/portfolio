const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { test } = require("node:test");
const { runInNewContext } = require("node:vm");

const script = readFileSync(new URL("../assets/portrait.js", `file://${__filename}`), "utf8");

function setup({ missing = false, keyboard = false } = {}) {
  const events = {};
  const attributes = {};
  const card = {
    getAttribute: (name) => attributes[name],
    setAttribute: (name, value) => { attributes[name] = value; },
    addEventListener: (name, callback) => { events[name] = callback; },
    matches: () => keyboard,
    contains: (target) => target === card,
  };
  runInNewContext(script, {
    document: {
      getElementById: () => missing ? null : card,
      addEventListener: (name, callback) => { events[`document:${name}`] = callback; },
    },
  });
  return { events, attributes, card };
}

test("hover reveals the full portrait and leaving collapses it", () => {
  const { events, attributes } = setup();
  assert.equal(attributes["data-enhanced"], "true");
  events.pointerenter({ pointerType: "mouse" });
  assert.equal(attributes["aria-expanded"], "true");
  assert.equal(attributes["aria-label"], "Hide full profile picture");
  events.pointerleave();
  assert.equal(attributes["aria-expanded"], "false");
});

test("touch taps toggle the reveal without emulating hover", () => {
  const { events, attributes } = setup();
  events.pointerenter({ pointerType: "touch" });
  events.focus();
  assert.equal(attributes["aria-expanded"], "false");
  events.click();
  assert.equal(attributes["aria-expanded"], "true");
  events.click();
  assert.equal(attributes["aria-expanded"], "false");
});

test("clicking a hovered portrait dismisses it instead of pinning it", () => {
  const { events, attributes } = setup();
  events.pointerenter({ pointerType: "mouse" });
  events.click();
  assert.equal(attributes["aria-expanded"], "false");
  events.pointerleave();
  assert.equal(attributes["aria-expanded"], "false");
});

test("keyboard focus reveals the portrait; Escape dismisses it", () => {
  const { events, attributes } = setup({ keyboard: true });
  events.focus();
  assert.equal(attributes["aria-expanded"], "true");
  events["document:keydown"]({ key: "Escape" });
  assert.equal(attributes["aria-expanded"], "false");
  events.focus();
  events.blur();
  assert.equal(attributes["aria-expanded"], "false");
});

test("a pinned portrait closes when tapping outside", () => {
  const { events, attributes, card } = setup();
  events.click();
  events["document:pointerdown"]({ target: card });
  assert.equal(attributes["aria-expanded"], "true");
  events["document:pointerdown"]({ target: {} });
  assert.equal(attributes["aria-expanded"], "false");
});

test("pages without a portrait do not throw", () => {
  assert.doesNotThrow(() => setup({ missing: true }));
});
