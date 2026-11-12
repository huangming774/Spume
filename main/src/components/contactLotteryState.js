export const getDrawDigits = (number) =>
  String(number).padStart(3, "0").split("");

export function createSettledDrawState({ prize, number }) {
  const shouldLoadWallpaper = prize?.id === "wallpaper";

  return {
    drawNumber: number,
    isDrawing: false,
    resultOpen: true,
    shouldLoadWallpaper,
    slotDigits: getDrawDigits(number),
    wallpaperLoadStatus: shouldLoadWallpaper ? "loading" : "idle",
  };
}

