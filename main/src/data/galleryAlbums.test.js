import test from "node:test";
import assert from "node:assert/strict";

import { galleryAlbums } from "./galleryAlbums.js";

test("serves gallery album images through the same-origin COS proxy", () => {
  const urls = galleryAlbums.flatMap((album) => [album.cover, ...album.images]);

  assert.ok(urls.length > 0);
  for (const url of urls) {
    assert.match(url, /^\/cos\//);
    assert.doesNotMatch(url, /cos\.ap-beijing\.myqcloud\.com/);
  }
});
