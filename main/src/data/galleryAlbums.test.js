import test from "node:test";
import assert from "node:assert/strict";

import { galleryAlbums } from "./galleryAlbums.js";

test("serves gallery album images from the R2 direct links", () => {
  const urls = galleryAlbums.flatMap((album) => [album.cover, ...album.images]);

  assert.ok(urls.length > 0);
  for (const url of urls) {
    assert.match(url, /^https:\/\/fs\.964643\.xyz\//);
    assert.doesNotMatch(url, /964643\.xyz\/\//);
    assert.doesNotMatch(url, /cos\.ap-beijing\.myqcloud\.com/);
  }
});
