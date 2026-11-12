import test from "node:test";
import assert from "node:assert/strict";

import { parseFriendQuickInput } from "./friendQuickInput.js";

test("recognizes friend fields from four shuffled lines", () => {
  const parsed = parseFriendQuickInput(`
https://anze.love/wp-content/uploads/2026/03/cropped-anze.jpg
愿得一人心.
https://anze.love
安泽的温馨小窝
`);

  assert.deepEqual(parsed, {
    name: "安泽的温馨小窝",
    desc: "愿得一人心.",
    url: "https://anze.love",
    avatar: "https://anze.love/wp-content/uploads/2026/03/cropped-anze.jpg",
  });
});

test("keeps a blog-like short text as name and the longer text as description", () => {
  const parsed = parseFriendQuickInput(`
记录技术与生活里的小火花
https://cdn.example/avatar.webp
Dream Blog
https://dream.example/
`);

  assert.equal(parsed.name, "Dream Blog");
  assert.equal(parsed.desc, "记录技术与生活里的小火花");
  assert.equal(parsed.url, "https://dream.example/");
  assert.equal(parsed.avatar, "https://cdn.example/avatar.webp");
});
