import test from "node:test";
import assert from "node:assert/strict";

import {
  getContinuousCompassRotation,
  getHeroIndexFromCompassRotation,
  getSnappedHeroCompassTarget,
} from "./heroCompassSelection.js";

const items = [
  { index: 1, angle: 0 },
  { index: 2, angle: 90 },
  { index: 3, angle: 180 },
  { index: 4, angle: 270 },
];

test("maps compass pointer rotation to the nearest hero card", () => {
  assert.equal(getHeroIndexFromCompassRotation(0, items), 1);
  assert.equal(getHeroIndexFromCompassRotation(80, items), 2);
  assert.equal(getHeroIndexFromCompassRotation(181, items), 3);
  assert.equal(getHeroIndexFromCompassRotation(-80, items), 4);
});

test("keeps boundary angles stable while wrapping around", () => {
  assert.equal(getHeroIndexFromCompassRotation(44, items), 1);
  assert.equal(getHeroIndexFromCompassRotation(46, items), 2);
  assert.equal(getHeroIndexFromCompassRotation(315, items), 1);
  assert.equal(getHeroIndexFromCompassRotation(316, items), 1);
});

test("snaps compass rotation to the selected hero card angle", () => {
  assert.deepEqual(getSnappedHeroCompassTarget(88, items), {
    index: 2,
    angle: 90,
  });
  assert.deepEqual(getSnappedHeroCompassTarget(-5, items), {
    index: 1,
    angle: 0,
  });
  assert.deepEqual(getSnappedHeroCompassTarget(266, items), {
    index: 4,
    angle: 270,
  });
  assert.deepEqual(getSnappedHeroCompassTarget(365, items), {
    index: 1,
    angle: 360,
  });
  assert.deepEqual(getSnappedHeroCompassTarget(-86, items), {
    index: 4,
    angle: -90,
  });
});

test("keeps compass rotation continuous across the wrap boundary", () => {
  assert.equal(getContinuousCompassRotation(350, 5), 365);
  assert.equal(getContinuousCompassRotation(10, 350), -10);
  assert.equal(getContinuousCompassRotation(725, 350), 710);
  assert.equal(getContinuousCompassRotation(-10, 5), 5);
});
