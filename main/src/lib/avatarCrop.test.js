import test from "node:test";
import assert from "node:assert/strict";

import {
  avatarExportType,
  clampSquareCrop,
  initialSquareCrop,
  minSquareCropSize,
  moveSquareCrop,
  outputEdge,
  resizeSquareCrop,
  setSquareCropSize,
} from "./avatarCrop.js";

test("starts with the largest centered square", () => {
  assert.deepEqual(initialSquareCrop(400, 200), { x: 100, y: 0, size: 200 });
  assert.deepEqual(initialSquareCrop(180, 320), { x: 0, y: 70, size: 180 });
});

test("moves the square and keeps it inside the image", () => {
  const crop = { x: 100, y: 0, size: 200 };
  assert.deepEqual(moveSquareCrop(crop, -80, 10, 400, 200), { x: 20, y: 0, size: 200 });
  assert.deepEqual(moveSquareCrop(crop, -500, 40, 400, 200), { x: 0, y: 0, size: 200 });
});

test("resizes from a corner without leaving the image", () => {
  const crop = { x: 100, y: 20, size: 80 };
  const grown = resizeSquareCrop(crop, "se", 40, 10, 400, 300);
  assert.equal(grown.x, 100);
  assert.equal(grown.y, 20);
  assert.equal(grown.size, 120);

  const shrunk = resizeSquareCrop(crop, "nw", 30, 20, 400, 300, 40);
  assert.equal(shrunk.size, 60);
  assert.equal(shrunk.x + shrunk.size, 180);
  assert.equal(shrunk.y + shrunk.size, 100);
});

test("changes size around the center and clamps to the short side", () => {
  const crop = { x: 50, y: 25, size: 50 };
  assert.deepEqual(setSquareCropSize(crop, 100, 200, 100), { x: 25, y: 0, size: 100 });
  assert.deepEqual(clampSquareCrop({ x: -10, y: 80, size: 500 }, 200, 100, 20), {
    x: 0,
    y: 0,
    size: 100,
  });
});

test("keeps a grabbable crop on screen and fits the saved edge to the avatar", () => {
  assert.equal(minSquareCropSize(2000, 3000, 500, 48), 192);
  assert.equal(outputEdge(2400), 1024);
  assert.equal(outputEdge(320), 320);
  assert.equal(avatarExportType("image/jpeg"), "image/jpeg");
  assert.equal(avatarExportType("image/png"), "image/png");
  assert.equal(avatarExportType("image/webp"), "image/png");
});
