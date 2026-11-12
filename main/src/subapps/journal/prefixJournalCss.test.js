import test from "node:test";
import assert from "node:assert/strict";
import { loadScopedJournalCss, prefixJournalCss, JOURNAL_PREFIX } from "./prefixJournalCss.js";

test("journal element selectors are prefixed and global tags are not", async () => {
  const css = await prefixJournalCss(`
    @import url("https://fonts.example/css");
    :root { --ink: #111; }
    * { box-sizing: border-box; letter-spacing: 0; }
    html { scroll-behavior: smooth; }
    body { background: #f8f9f8; color: #343b3d; }
    button, a { color: inherit; }
    button { cursor: pointer; }
    @keyframes rise { from { opacity: 0; } to { opacity: 1; } }
    @media (max-width: 600px) {
      body { margin: 0; }
      .card { color: red; }
    }
  `);

  assert.match(css, /@import url\("https:\/\/fonts\.example\/css"\)/);
  assert.match(css, new RegExp(`${JOURNAL_PREFIX}\\s*\\{[^}]*--ink:\\s*#111`));
  assert.match(css, new RegExp(`${JOURNAL_PREFIX} \\*`));
  assert.match(css, new RegExp(`${JOURNAL_PREFIX} button`));
  assert.match(css, new RegExp(`${JOURNAL_PREFIX} a`));
  assert.match(css, new RegExp(`${JOURNAL_PREFIX}\\s*\\{[^}]*background:\\s*#f8f9f8`));
  assert.match(css, /@keyframes rise\s*\{\s*from\s*\{/);
  assert.match(css, new RegExp(`@media[^{]+\\{[^}]*${JOURNAL_PREFIX}\\s*\\{[^}]*margin:\\s*0`));
  assert.match(css, new RegExp(`${JOURNAL_PREFIX} \\.card`));
  assert.doesNotMatch(css, /(^|[}{;])\s*(html|body|:root|button|a|\*)\s*[,{]/);
});

test("the journal stylesheet scopes archive element rules under the journal root", async () => {
  const css = await loadScopedJournalCss();
  assert.match(css, new RegExp(`${JOURNAL_PREFIX}\\s*\\{[^}]*--archive-paper`));
  assert.match(css, new RegExp(`${JOURNAL_PREFIX} button`));
  assert.match(css, new RegExp(`${JOURNAL_PREFIX} \\.hljs`));
  assert.match(css, /@keyframes momentIn\s*\{[\s\S]*?from\s*\{/);
  assert.doesNotMatch(css, /(^|[}{;])\s*(html|body|:root|button|a|\*)\s*[,{]/);
});
