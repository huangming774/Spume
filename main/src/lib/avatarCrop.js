export function initialSquareCrop(width, height) {
  const size = Math.min(width, height);
  return clampSquareCrop(
    {
      x: (width - size) / 2,
      y: (height - size) / 2,
      size,
    },
    width,
    height,
  );
}

export function clampSquareCrop(crop, width, height, minSize = 1) {
  const maxSize = Math.max(1, Math.min(width, height));
  const floor = Math.min(Math.max(1, minSize), maxSize);
  const size = Math.min(Math.max(floor, crop.size), maxSize);
  return {
    x: Math.min(Math.max(0, crop.x), width - size),
    y: Math.min(Math.max(0, crop.y), height - size),
    size,
  };
}

export function moveSquareCrop(crop, dx, dy, width, height, minSize = 1) {
  return clampSquareCrop({ ...crop, x: crop.x + dx, y: crop.y + dy }, width, height, minSize);
}

export function resizeSquareCrop(crop, handle, dx, dy, width, height, minSize = 1) {
  const right = crop.x + crop.size;
  const bottom = crop.y + crop.size;
  let size = crop.size;
  if (handle === "se") size += Math.max(dx, dy);
  else if (handle === "nw") size += Math.max(-dx, -dy);
  else if (handle === "ne") size += Math.max(dx, -dy);
  else if (handle === "sw") size += Math.max(-dx, dy);

  let next = { x: crop.x, y: crop.y, size };
  if (handle === "nw") next = { x: right - size, y: bottom - size, size };
  else if (handle === "ne") next = { x: crop.x, y: bottom - size, size };
  else if (handle === "sw") next = { x: right - size, y: crop.y, size };
  return clampSquareCrop(next, width, height, minSize);
}

export function setSquareCropSize(crop, size, width, height, minSize = 1) {
  const centerX = crop.x + crop.size / 2;
  const centerY = crop.y + crop.size / 2;
  return clampSquareCrop(
    {
      x: centerX - size / 2,
      y: centerY - size / 2,
      size,
    },
    width,
    height,
    minSize,
  );
}

export function minSquareCropSize(width, height, displayWidth, minScreenPx = 48) {
  const shortSide = Math.min(width, height);
  if (!displayWidth || shortSide <= minScreenPx) return Math.max(1, shortSide);
  return Math.min(shortSide, Math.max(1, minScreenPx / (displayWidth / width)));
}

export function outputEdge(cropSize, maxEdge = 1024) {
  return Math.max(1, Math.min(maxEdge, Math.round(cropSize)));
}

export function avatarExportType(fileType) {
  return fileType === "image/jpeg" ? "image/jpeg" : "image/png";
}

export function renderAvatarFile(image, crop, fileType = "image/png") {
  const edge = outputEdge(crop.size);
  const canvas = document.createElement("canvas");
  canvas.width = edge;
  canvas.height = edge;
  const context = canvas.getContext("2d");
  if (!context) return Promise.reject(new Error("无法裁切这张图片"));
  context.imageSmoothingEnabled = true;
  context.imageSmoothingQuality = "high";
  context.drawImage(image, crop.x, crop.y, crop.size, crop.size, 0, 0, edge, edge);
  const type = avatarExportType(fileType);
  return new Promise((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (!blob) {
          reject(new Error("无法裁切这张图片"));
          return;
        }
        const extension = type === "image/jpeg" ? "jpg" : "png";
        resolve(new File([blob], `avatar.${extension}`, { type }));
      },
      type,
      0.92,
    );
  });
}
