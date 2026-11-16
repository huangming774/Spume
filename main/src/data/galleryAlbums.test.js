import test from "node:test";
import assert from "node:assert/strict";

import { galleryAlbums } from "./galleryAlbums.js";

test("starts with no glimmer albums", () => {
  assert.deepEqual(galleryAlbums, []);
});
