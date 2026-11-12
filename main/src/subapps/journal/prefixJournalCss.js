import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import postcss from "postcss";
import prefixSelector from "postcss-prefix-selector";

const journalDir = dirname(fileURLToPath(import.meta.url));

export const JOURNAL_ROOT_CLASS = "journal-root";
export const JOURNAL_PREFIX = `.${JOURNAL_ROOT_CLASS}`;

const sourceFiles = [
  join(journalDir, "site-pages.css"),
  join(journalDir, "src", "styles", "archive.css"),
  join(journalDir, "..", "..", "..", "node_modules", "highlight.js", "styles", "atom-one-dark.css"),
];

export async function prefixJournalCss(css) {
  const result = await postcss([
    prefixSelector({ prefix: JOURNAL_PREFIX }),
  ]).process(css, { from: join(journalDir, "journal.css") });
  return `${result.css}\n${JOURNAL_PREFIX}{min-height:100vh;}\n`;
}

export async function loadScopedJournalCss() {
  const css = sourceFiles.map((file) => readFileSync(file, "utf8")).join("\n");
  return prefixJournalCss(css);
}
