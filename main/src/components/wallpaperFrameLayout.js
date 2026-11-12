export const DEFAULT_WALLPAPER_ASPECT_RATIO = 16 / 9;

export function getWallpaperCardStyle(aspectRatio = DEFAULT_WALLPAPER_ASPECT_RATIO) {
  const safeAspectRatio =
    Number.isFinite(aspectRatio) && aspectRatio > 0
      ? aspectRatio
      : DEFAULT_WALLPAPER_ASPECT_RATIO;

  return {
    "--wallpaper-aspect-ratio": safeAspectRatio,
    width: "fit-content",
    maxWidth: "94vw",
  };
}
