export const normalizeCompassAngle = (angle) => ((angle % 360) + 360) % 360;

export const getCompassAngleDistance = (angle, targetAngle) =>
  Math.abs(((normalizeCompassAngle(angle - targetAngle) + 180) % 360) - 180);

export const getContinuousCompassRotation = (previousRotation, nextAngle) => {
  const normalizedPrevious = normalizeCompassAngle(previousRotation);
  let delta = normalizeCompassAngle(nextAngle) - normalizedPrevious;

  if (delta > 180) delta -= 360;
  if (delta < -180) delta += 360;

  return previousRotation + delta;
};

export const getHeroIndexFromCompassRotation = (rotation, items) => {
  if (!Array.isArray(items) || items.length === 0) return null;

  return items.reduce((closestItem, item) => {
    const distance = getCompassAngleDistance(rotation, item.angle);
    if (!closestItem || distance < closestItem.distance) {
      return { index: item.index, distance };
    }
    return closestItem;
  }, null).index;
};

export const getSnappedHeroCompassTarget = (rotation, items) => {
  if (!Array.isArray(items) || items.length === 0) return null;

  const closestItem = items.reduce((closestItem, item) => {
    const distance = getCompassAngleDistance(rotation, item.angle);
    if (!closestItem || distance < closestItem.distance) {
      return { index: item.index, angle: item.angle, distance };
    }
    return closestItem;
  }, null);

  return {
    index: closestItem.index,
    angle: getContinuousCompassRotation(rotation, closestItem.angle),
  };
};
