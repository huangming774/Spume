import test from "node:test";
import assert from "node:assert/strict";

import { createSettledDrawState } from "./contactLotteryState.js";

test("settles wallpaper draws before the wallpaper request resolves", () => {
  const state = createSettledDrawState({
    prize: { id: "wallpaper", label: "Wallpaper Gift" },
    number: 876,
  });

  assert.equal(state.isDrawing, false);
  assert.equal(state.resultOpen, true);
  assert.equal(state.wallpaperLoadStatus, "loading");
  assert.equal(state.shouldLoadWallpaper, true);
  assert.deepEqual(state.slotDigits, ["8", "7", "6"]);
});

