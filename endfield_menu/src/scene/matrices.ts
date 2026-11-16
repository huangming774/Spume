import { Matrix4 } from "three";

/** Column-major CSS matrix3d → Three matrix. Same Y flip as three.js CSS3DRenderer (MIT). */

const EPS = 1e-4;

export function cssNum(value: number): number {
  const rounded = Math.round(value * 1000) / 1000;
  return Math.abs(rounded) < EPS ? 0 : rounded;
}

export function cameraCSSMatrix(elements: number[]): string {
  const e = elements;
  return (
    "matrix3d(" +
    [e[0], -e[1], e[2], e[3], e[4], -e[5], e[6], e[7], e[8], -e[9], e[10], e[11], e[12], -e[13], e[14], e[15]]
      .map(cssNum)
      .join(",") +
    ")"
  );
}

export function objectCSSMatrix(elements: number[]): string {
  const e = elements;
  const body =
    "matrix3d(" +
    [e[0], e[1], e[2], e[3], -e[4], -e[5], -e[6], -e[7], e[8], e[9], e[10], e[11], e[12], e[13], e[14], e[15]]
      .map(cssNum)
      .join(",") +
    ")";
  return "translate(-50%,-50%)" + body;
}

const S = new Matrix4().makeScale(1, -1, 1);
const scratch = new Matrix4();

/**
 * A CSS matrix measured in stage pixels (y down, origin at the stage's top-left)
 * becomes a Three matrix in the menu group (y up, origin at the stage centre).
 */
export function stageMatrixToGroup(columnMajor: number[], out: Matrix4): Matrix4 {
  scratch.fromArray(columnMajor);
  out.multiplyMatrices(S, scratch);
  out.multiply(S);
  out.elements[12] -= 960;
  out.elements[13] += 540;
  return out;
}
