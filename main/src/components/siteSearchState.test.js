import test from "node:test";
import assert from "node:assert/strict";
import { isSearchable, normalizeHits } from "./siteSearchState.js";

test("只有去空白后非空的查询才发请求", () => {
  assert.equal(isSearchable("机器学习"), true);
  assert.equal(isSearchable("  建站  "), true);
  assert.equal(isSearchable(""), false);
  assert.equal(isSearchable("   "), false);
  assert.equal(isSearchable(undefined), false);
});

test("命中清洗：丢掉缺标题/地址的行，摘要截到 80 字", () => {
  const hits = normalizeHits([
    { title: " 你好 ", url: " /blog/hello ", excerpt: "x".repeat(120) },
    { title: "", url: "/blog/empty" },
    { title: "没地址" },
    null,
    { title: "正常", url: "/build/post/ok", excerpt: 42 },
  ]);
  assert.equal(hits.length, 2);
  assert.deepEqual(hits[0], {
    title: "你好",
    url: "/blog/hello",
    excerpt: "x".repeat(80),
  });
  assert.equal(hits[1].excerpt, "");
});

test("不是数组或空数组都得到空列表", () => {
  assert.deepEqual(normalizeHits(undefined), []);
  assert.deepEqual(normalizeHits("oops"), []);
  assert.deepEqual(normalizeHits([]), []);
});
