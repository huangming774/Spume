import test from "node:test";
import assert from "node:assert/strict";

import {
  getOwnerGalleryAlbumSelection,
  ownerCustomGalleryAlbumValue,
  ownerGalleryAlbumOptions,
  resolveOwnerGalleryAlbum,
} from "./ownerGalleryAlbums.js";

test("ownerGalleryAlbumOptions include existing albums and custom option", () => {
  assert.ok(ownerGalleryAlbumOptions.length >= 4);
  assert.equal(ownerGalleryAlbumOptions[0].value, "misaka");
  assert.equal(ownerGalleryAlbumOptions[0].label, "御坂美琴");
  assert.equal(
    ownerGalleryAlbumOptions[ownerGalleryAlbumOptions.length - 1].value,
    ownerCustomGalleryAlbumValue,
  );
});

test("resolveOwnerGalleryAlbum returns custom album name when selected", () => {
  assert.equal(resolveOwnerGalleryAlbum(ownerCustomGalleryAlbumValue, " 自定义相册 "), "自定义相册");
  assert.equal(resolveOwnerGalleryAlbum("tangwulin", "ignored"), "tangwulin");
});

test("getOwnerGalleryAlbumSelection preserves existing album id and title", () => {
  assert.deepEqual(getOwnerGalleryAlbumSelection("misaka", ""), {
    albumId: "misaka",
    albumTitle: "御坂美琴",
  });
});

test("getOwnerGalleryAlbumSelection uses custom album name as id and title", () => {
  assert.deepEqual(getOwnerGalleryAlbumSelection(ownerCustomGalleryAlbumValue, " 夏日相册 "), {
    albumId: "",
    albumTitle: "夏日相册",
  });
});
