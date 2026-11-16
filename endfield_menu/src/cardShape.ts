import { CARD_SHAPES, type CardShape } from "./data";

const PLATE_LIP = 2.35;
const PLATE_GROOVE = 2.9;

type EdgeShape = { top: number[]; bottom: number[]; left: number[]; right: number[] };

/** Outline of a card face, grown by `pad` px, with corners rounded over `rc` px. */
export function shapePath(s: EdgeShape, pad: number, rc: number): string {
  const lineY = (c: number[], x: number) => c[0] + c[1] * x;
  const sideX = (q: number[], y: number) => q[0] + q[1] * y + q[2] * y * y;
  const slope = (q: number[], y: number) => q[1] + 2 * q[2] * y;
  const ym = (lineY(s.top, 250) + lineY(s.bottom, 250)) / 2;
  const top = [s.top[0] - pad * Math.hypot(1, s.top[1]), s.top[1]];
  const bottom = [s.bottom[0] + pad * Math.hypot(1, s.bottom[1]), s.bottom[1]];
  const left = [s.left[0] - pad * Math.hypot(1, slope(s.left, ym)), s.left[1], s.left[2]];
  const right = [s.right[0] + pad * Math.hypot(1, slope(s.right, ym)), s.right[1], s.right[2]];

  function corner(edge: number[], side: number[]) {
    let y = lineY(edge, 250);
    for (let k = 0; k < 40; k += 1) y = lineY(edge, sideX(side, y));
    return [sideX(side, y), y];
  }
  function along(p: number[], edge: number[], dir: number) {
    const n = Math.hypot(1, edge[1]);
    return [p[0] + (dir * rc) / n, p[1] + (dir * rc * edge[1]) / n];
  }
  function down(p: number[], side: number[], dir: number) {
    const y = p[1] + (dir * rc) / Math.hypot(1, slope(side, p[1]));
    return [sideX(side, y), y];
  }
  function ctrl(side: number[], y1: number, y2: number) {
    return [sideX(side, y1) + (slope(side, y1) * (y2 - y1)) / 2, (y1 + y2) / 2];
  }
  const f = (p: number[]) => p[0].toFixed(1) + " " + p[1].toFixed(1);

  const TL = corner(top, left);
  const TR = corner(top, right);
  const BR = corner(bottom, right);
  const BL = corner(bottom, left);
  const t1 = along(TL, top, 1);
  const t2 = along(TR, top, -1);
  const r1 = down(TR, right, 1);
  const r2 = down(BR, right, -1);
  const b1 = along(BR, bottom, -1);
  const b2 = along(BL, bottom, 1);
  const l1 = down(BL, left, -1);
  const l2 = down(TL, left, 1);
  return (
    "M" + f(t1) + " L" + f(t2) + " Q" + f(TR) + " " + f(r1) +
    " Q" + f(ctrl(right, r1[1], r2[1])) + " " + f(r2) +
    " Q" + f(BR) + " " + f(b1) + " L" + f(b2) + " Q" + f(BL) + " " + f(l1) +
    " Q" + f(ctrl(left, l1[1], l2[1])) + " " + f(l2) +
    " Q" + f(TL) + " " + f(t1) + " Z"
  );
}

export function cardOutline(index: number, pad = 0): string {
  return shapePath(CARD_SHAPES[index], pad, 12);
}

export function platePath(index: number): string {
  const s: CardShape = CARD_SHAPES[index];
  const plate: EdgeShape = {
    top: s.plate.top,
    left: s.plate.left,
    right: [s.right[0] + 30, s.right[1], s.right[2]],
    bottom: [s.bottom[0] + 30, s.bottom[1]],
  };
  return [0, PLATE_LIP, PLATE_LIP + PLATE_GROOVE]
    .map((inset) => shapePath(plate, -inset, 12 - inset))
    .join(" ");
}
