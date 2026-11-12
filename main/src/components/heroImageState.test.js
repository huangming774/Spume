import test from "node:test";
import assert from "node:assert/strict";

import {
  getNextHeroIndex,
  getHeroIndexAfterClick,
  isHeroImageReady,
} from "./heroImageState.js";

test("keeps the current hero when the preview target is not loaded yet", () => {
  const loadedHeroIndexes = new Set([1, 2]);

  assert.equal(getNextHeroIndex(2, 4), 3);
  assert.equal(isHeroImageReady(3, loadedHeroIndexes), false);
  assert.equal(getHeroIndexAfterClick(2, loadedHeroIndexes, 4), 2);
});

test("advances to the preview target after that hero image is loaded", () => {
  const loadedHeroIndexes = new Set([1, 2, 3]);

  assert.equal(getHeroIndexAfterClick(2, loadedHeroIndexes, 4), 3);
});

test("wraps from the last hero back to the first loaded hero", () => {
  const loadedHeroIndexes = new Set([1, 2, 3, 4]);

  assert.equal(getHeroIndexAfterClick(4, loadedHeroIndexes, 4), 1);
});
