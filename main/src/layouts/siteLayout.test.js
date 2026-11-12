import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const siteLayoutSource = readFileSync(
  new URL("./SiteLayout.jsx", import.meta.url),
  "utf8"
);

test("keeps the site shell from becoming a vertical scroll container", () => {
  assert.match(siteLayoutSource, /overflow-x-clip/);
  assert.doesNotMatch(siteLayoutSource, /overflow-x-hidden/);
});
