import { ART_DEFS, BUTTON_ART } from "./buttonArt";

const MARKUP =
  ART_DEFS +
  Object.entries(BUTTON_ART)
    .map(([id, html]) => `<g id="art-${id}">${html}</g>`)
    .join("");

/** Hidden sprite so cards, tiles and badges can `<use href="#art-…">`. */
export function ArtSprite() {
  return (
    <svg width="0" height="0" style={{ position: "absolute" }} aria-hidden="true">
      <defs dangerouslySetInnerHTML={{ __html: MARKUP }} />
    </svg>
  );
}
