export const getNextHeroIndex = (currentIndex, totalImages) =>
  (currentIndex % totalImages) + 1;

export const isHeroImageReady = (index, loadedHeroIndexes) =>
  loadedHeroIndexes instanceof Set && loadedHeroIndexes.has(index);

export const getHeroIndexAfterClick = (
  currentIndex,
  loadedHeroIndexes,
  totalImages
) => {
  const nextIndex = getNextHeroIndex(currentIndex, totalImages);
  return isHeroImageReady(nextIndex, loadedHeroIndexes)
    ? nextIndex
    : currentIndex;
};

export const areHeroImagesReady = (indexes, loadedHeroIndexes) =>
  indexes.every((index) => isHeroImageReady(index, loadedHeroIndexes));
