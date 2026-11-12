import test from "node:test";
import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { parseFrontMatter, renderPostBody } from "./renderPost.js";

const postsDir = join(dirname(fileURLToPath(import.meta.url)), "../../../../blog/source/_posts");

test("article markdown renders without leftover butterfly tags", () => {
  const files = readdirSync(postsDir).filter((name) => name.endsWith(".md"));
  assert.ok(files.length >= 20);
  for (const file of files) {
    const { body } = parseFrontMatter(readFileSync(join(postsDir, file), "utf8"));
    const html = renderPostBody(body);
    const visible = html
      .replace(/<pre[\s\S]*?<\/pre>/g, "")
      .replace(/<code[\s\S]*?<\/code>/g, "");
    assert.doesNotMatch(visible, /\{%/, file);
  }
});
