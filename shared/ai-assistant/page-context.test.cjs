const test = require("node:test");
const assert = require("node:assert/strict");

const {
  buildPageContext,
  createChatPayload,
  normalizeWhitespace,
} = require("./page-context.js");

test("normalizes noisy visible page text into a compact page context", () => {
  const context = buildPageContext({
    pageUrl: "https://taozhiyy.top/about",
    pageTitle: "About Taozhiyy",
    pagePath: "/about",
    language: "zh-CN",
    siteSection: "main",
    headings: ["About", "", "  技术栈  "],
    visibleText: "  关于 桃之夭夭\n\n这里介绍博客由来、技术栈和联系方式。  ",
  });

  assert.equal(context.pagePath, "/about");
  assert.deepEqual(context.headings, ["About", "技术栈"]);
  assert.equal(
    context.visibleText,
    "关于 桃之夭夭 这里介绍博客由来、技术栈和联系方式。"
  );
});

test("chat payload includes structured page context as evidence for the bot", () => {
  const context = buildPageContext({
    pageUrl: "https://taozhiyy.top/about",
    pageTitle: "About Taozhiyy",
    pagePath: "/about",
    visibleText: "这是关于页面的真实正文。",
  });

  const payload = createChatPayload("这个页面有什么？", context);

  assert.equal(payload.message, "这个页面有什么？");
  assert.equal(payload.pageUrl, "https://taozhiyy.top/about");
  assert.equal(payload.pageTitle, "About Taozhiyy");
  assert.equal(payload.pageContext.visibleText, "这是关于页面的真实正文。");
});

test("normalizeWhitespace removes repeated whitespace without losing words", () => {
  assert.equal(normalizeWhitespace(" A\n\nB\t C  "), "A B C");
});
