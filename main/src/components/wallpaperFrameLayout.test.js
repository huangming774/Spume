import test from "node:test";
import assert from "node:assert/strict";

import {
  DEFAULT_WALLPAPER_ASPECT_RATIO,
  getWallpaperCardStyle,
} from "./wallpaperFrameLayout.js";

test("getWallpaperCardStyle keeps the wallpaper card sized to its media content", () => {
  const style = getWallpaperCardStyle(0.72);

  assert.equal(style.width, "fit-content");
  assert.equal(style.maxWidth, "94vw");
  assert.equal(style["--wallpaper-aspect-ratio"], 0.72);
});

test("getWallpaperCardStyle falls back to the default ratio for invalid input", () => {
  const style = getWallpaperCardStyle(0);

  assert.equal(
    style["--wallpaper-aspect-ratio"],
    DEFAULT_WALLPAPER_ASPECT_RATIO
  );
});
