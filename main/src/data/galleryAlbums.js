export const galleryAlbums = [
];

export const getGalleryAlbum = (id) =>
  galleryAlbums.find((album) => album.id === id);
