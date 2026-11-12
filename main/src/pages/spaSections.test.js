import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const read = (path) => readFileSync(new URL(path, import.meta.url), "utf8");

test("articles, journal and cosmos stay inside the main router", () => {
  const app = read("../App.jsx");
  const nav = read("../components/Navbar.jsx");
  assert.match(app, /path="blog"/);
  assert.match(app, /path="blog\/\*"/);
  assert.match(app, /path="build\/\*"/);
  assert.match(app, /path="projects\/\*"/);
  assert.match(nav, /label: "文章", to: "\/blog"/);
  assert.match(nav, /label: "建站日志", to: "\/build"/);
  assert.match(nav, /label: "星空馆", to: "\/projects"/);
  assert.doesNotMatch(nav, /href: "\/blog\/"|href: "\/build\/"|href: "\/projects\/"/);
});
