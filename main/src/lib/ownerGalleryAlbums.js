import { galleryAlbums } from "../data/galleryAlbums.js";

export const ownerCustomGalleryAlbumValue = "__custom__";

export const ownerGalleryAlbumOptions = [
  ...galleryAlbums.map((album) => ({
    value: album.id,
    label: album.title,
  })),
  {
    value: ownerCustomGalleryAlbumValue,
    label: "自定义",
  },
];

export function resolveOwnerGalleryAlbum(selectedAlbum, customAlbum = "") {
  if (selectedAlbum === ownerCustomGalleryAlbumValue) {
    return customAlbum.trim();
  }
  return selectedAlbum.trim();
}

export function getOwnerGalleryAlbumSelection(selectedAlbum, customAlbum = "") {
  const albumId = resolveOwnerGalleryAlbum(selectedAlbum, customAlbum);
  if (!albumId) {
    return { albumId: "", albumTitle: "" };
  }

  if (selectedAlbum === ownerCustomGalleryAlbumValue) {
    return { albumId: "", albumTitle: albumId };
  }

  const option = ownerGalleryAlbumOptions.find((item) => item.value === albumId);
  return {
    albumId,
    albumTitle: option?.label || albumId,
  };
}
