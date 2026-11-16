import { readFileSync } from "node:fs";
import postcss from "postcss";
import prefixSelector from "postcss-prefix-selector";

export const WHERE_ROOT_CLASS = "where-menu";
export const WHERE_PREFIX = `.${WHERE_ROOT_CLASS}`;

const PAGE_SELECTORS = new Set(["html", "body", "#root", "html, body, #root"]);

export async function loadWhereMenuCss(file) {
  const css = readFileSync(file, "utf8");
  const result = await postcss([
    prefixSelector({
      prefix: WHERE_PREFIX,
      transform(prefix, selector, prefixed) {
        const trimmed = selector.trim();
        if (trimmed === ":root" || PAGE_SELECTORS.has(trimmed)) return prefix;
        return prefixed;
      },
    }),
  ]).process(css, { from: file });
  return `${result.css}
${WHERE_PREFIX} {
  position: fixed;
  inset: 0;
  z-index: 80;
  overflow: hidden;
  background: #000;
}
${WHERE_PREFIX},
${WHERE_PREFIX} .app,
${WHERE_PREFIX} .wallpaper,
${WHERE_PREFIX} canvas.gl {
  background-color: #000 !important;
  background-image: none !important;
}
${WHERE_PREFIX} .dim {
  background: #000 !important;
  opacity: 1 !important;
}
${WHERE_PREFIX}[data-shown="0"] {
  visibility: hidden;
  pointer-events: none;
}
${WHERE_PREFIX} .app[data-phase="enter"] .card,
${WHERE_PREFIX} .app[data-phase="enter"] .row,
${WHERE_PREFIX} .app[data-phase="enter"] .news,
${WHERE_PREFIX} .app[data-phase="enter"] .badge,
${WHERE_PREFIX} .app[data-phase="enter"] .sys,
${WHERE_PREFIX} .app[data-phase="enter"] .power,
${WHERE_PREFIX} .app[data-phase="enter"] .close,
${WHERE_PREFIX} .app[data-phase="enter"] .topbar,
${WHERE_PREFIX} .app[data-phase="enter"] .energy,
${WHERE_PREFIX} .app[data-phase="enter"] .disc-btns,
${WHERE_PREFIX} .app[data-phase="enter"] .level,
${WHERE_PREFIX} .app[data-phase="enter"] .cables,
${WHERE_PREFIX} .app[data-phase="enter"] .belt {
  animation-duration: 0.16s;
  animation-delay: 0s;
}
`;
}
