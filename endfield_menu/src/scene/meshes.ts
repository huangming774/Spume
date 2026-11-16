import {
  BufferAttribute,
  BufferGeometry,
  CircleGeometry,
  DoubleSide,
  Group,
  InstancedMesh,
  Mesh,
  MeshBasicMaterial,
  Object3D,
  PlaneGeometry,
  RingGeometry,
  ShaderMaterial,
  Vector3,
} from "three";
import { FLOOR_STROKES } from "../data";

const DEG = Math.PI / 180;
const SVG_SCALE = 0.96;
export const WELL_R = 447;

const discVert = /* glsl */ `
varying vec2 vP;
void main() {
  vP = position.xy / ${WELL_R.toFixed(1)};
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;

const discFrag = /* glsl */ `
precision highp float;
varying vec2 vP;
uniform float uTime;
uniform float uReveal;
uniform float uAlpha;
void main() {
  float r = length(vP);
  vec3 col = vec3(0.033, 0.038, 0.045);
  vec2 g = abs(fract(vP * 7.5) - 0.5);
  float grid = 1.0 - smoothstep(0.0, 0.045, min(g.x, g.y));
  col += vec3(0.45, 0.75, 1.0) * grid * 0.09;
  vec2 cell = fract(vP * 10.0) - 0.5;
  float dots = smoothstep(0.07, 0.02, length(cell));
  col += dots * 0.045;
  float scan = 0.72 + 0.28 * sin((vP.y + 1.0) * 140.0 + uTime * 1.4);
  col += scan * 0.03;
  float ang = atan(vP.y, vP.x);
  float sweep = fract(ang / 6.2831853 - uTime * 0.045);
  float beam = smoothstep(0.16, 0.0, sweep);
  col += vec3(0.35, 0.75, 1.0) * beam * (1.0 - r) * 0.22;
  col *= smoothstep(1.02, 0.62, r);
  vec2 fromTop = vP - vec2(0.0, 0.85);
  float rad = mix(0.0, 2.5, uReveal);
  float mask = smoothstep(rad, rad - 0.28, length(fromTop));
  float alpha = mask * uAlpha;
  if (alpha < 0.01) discard;
  gl_FragColor = vec4(col, alpha);
}
`;

function arcMesh(radius: number, a0: number, a1: number, width: number, color: number, opacity = 1): Mesh {
  const start = -a1 * DEG;
  const length = (a1 - a0) * DEG;
  const segs = Math.max(12, Math.ceil((length / (Math.PI * 2)) * 160));
  const geo = new RingGeometry(radius - width / 2, radius + width / 2, segs, 1, start, length);
  const mat = new MeshBasicMaterial({
    color,
    transparent: true,
    opacity,
    side: DoubleSide,
    depthWrite: false,
  });
  const mesh = new Mesh(geo, mat);
  mesh.renderOrder = 2;
  return mesh;
}

function fullRing(inner: number, outer: number, color: number, opacity: number): Mesh {
  const geo = new RingGeometry(inner, outer, 128, 1);
  const mat = new MeshBasicMaterial({
    color,
    transparent: true,
    opacity,
    side: DoubleSide,
    depthWrite: false,
  });
  const mesh = new Mesh(geo, mat);
  mesh.renderOrder = 1;
  return mesh;
}

function flagGeometry(): BufferGeometry {
  const geo = new BufferGeometry();
  geo.setAttribute(
    "position",
    new BufferAttribute(new Float32Array([
      0, 0, 0,
      16, 46, 0,
      -16, 46, 0,
    ]), 3),
  );
  return geo;
}

export type DiscKit = {
  group: Group;
  flags: Group;
  uniforms: { uTime: { value: number }; uReveal: { value: number }; uAlpha: { value: number } };
  skel: Group;
  skelMat: MeshBasicMaterial;
};

export function buildDisc(): DiscKit {
  const group = new Group();
  const uniforms = {
    uTime: { value: 0 },
    uReveal: { value: 0 },
    uAlpha: { value: 0 },
  };
  const well = new Mesh(
    new CircleGeometry(WELL_R, 96),
    new ShaderMaterial({
      uniforms,
      vertexShader: discVert,
      fragmentShader: discFrag,
      transparent: true,
      depthWrite: false,
    }),
  );
  well.position.z = -130;
  well.renderOrder = 0;
  group.add(well);

  const rim = fullRing(570 * SVG_SCALE, 677 * SVG_SCALE, 0x0e1012, 0.92);
  rim.position.z = -112;
  group.add(rim);

  const amber = new Group();
  amber.position.z = -60;
  amber.add(arcMesh(466 * SVG_SCALE, 44, 137, 1.3, 0xf08d36, 0.95));
  amber.add(arcMesh(466 * SVG_SCALE, 160, 380, 1.3, 0xf08d36, 0.95));
  amber.add(arcMesh(466 * SVG_SCALE, 20, 44, 1.8, 0xf59c48, 0.85));
  amber.add(arcMesh(466 * SVG_SCALE, 137, 160, 1.8, 0xf59c48, 0.85));
  group.add(amber);

  [472, 476, 480, 484].forEach((radius, i) => {
    const groove = fullRing(radius * SVG_SCALE - 0.4, radius * SVG_SCALE + 0.4, 0xffffff, i % 2 ? 0.06 : 0.1);
    groove.position.z = 10 + i * 0.4;
    group.add(groove);
  });

  const white = new Group();
  white.position.z = 50;
  for (const [a0, a1] of [[-40.5, -10], [-4, 44], [138, 168], [174, 204], [210, 234]] as Array<[number, number]>) {
    white.add(arcMesh(492.5 * SVG_SCALE, a0, a1, 9.4, 0xffffff, 0.94));
  }
  group.add(white);

  const flags = new Group();
  const flagMesh = new InstancedMesh(
    flagGeometry(),
    new MeshBasicMaterial({ color: 0xffde00, side: DoubleSide, depthWrite: false }),
    5,
  );
  flagMesh.renderOrder = 3;
  const dummy = new Object3D();
  const flagR = 462 * SVG_SCALE;
  [-101, -162, -37, 29, 141].forEach((deg, i) => {
    const a = deg * DEG;
    dummy.position.set(Math.cos(a) * flagR, -Math.sin(a) * flagR, 55);
    dummy.rotation.set(0, 0, -a - Math.PI / 2);
    dummy.updateMatrix();
    flagMesh.setMatrixAt(i, dummy.matrix);
  });
  flagMesh.instanceMatrix.needsUpdate = true;
  flagMesh.frustumCulled = false;
  flags.add(flagMesh);
  group.add(flags);

  const skelMat = new MeshBasicMaterial({
    color: 0x54c0e6,
    transparent: true,
    opacity: 0,
    depthWrite: false,
    side: DoubleSide,
  });
  const skel = new Group();
  skel.position.z = -62;
  skel.add(new Mesh(new RingGeometry(WELL_R - 10, WELL_R + 1, 96, 1), skelMat));
  const hx = new Mesh(new PlaneGeometry(WELL_R * 1.15, 1.6), skelMat);
  const hy = new Mesh(new PlaneGeometry(1.6, WELL_R * 1.15), skelMat);
  skel.add(hx, hy);
  group.add(skel);

  return { group, flags, uniforms, skel, skelMat };
}

function samplePath(d: string, steps: number): Vector3[] {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", d);
  svg.appendChild(path);
  svg.setAttribute("width", "0");
  svg.setAttribute("height", "0");
  svg.style.position = "absolute";
  svg.style.left = "-9999px";
  document.body.appendChild(svg);
  const len = path.getTotalLength();
  const pts: Vector3[] = [];
  const n = Math.max(2, steps);
  for (let i = 0; i < n; i += 1) {
    const p = path.getPointAtLength((len * i) / (n - 1));
    pts.push(new Vector3(p.x - 960, -(p.y - 540), -37.7));
  }
  svg.remove();
  return pts;
}

function ribbon(points: Vector3[], width: number): BufferGeometry {
  const positions: number[] = [];
  const half = width / 2;
  for (let i = 0; i < points.length - 1; i += 1) {
    const a = points[i];
    const b = points[i + 1];
    const dx = b.x - a.x;
    const dy = b.y - a.y;
    const len = Math.hypot(dx, dy) || 1;
    const nx = (-dy / len) * half;
    const ny = (dx / len) * half;
    positions.push(
      a.x + nx, a.y + ny, a.z,
      a.x - nx, a.y - ny, a.z,
      b.x + nx, b.y + ny, b.z,
      a.x - nx, a.y - ny, a.z,
      b.x - nx, b.y - ny, b.z,
      b.x + nx, b.y + ny, b.z,
    );
  }
  const geo = new BufferGeometry();
  geo.setAttribute("position", new BufferAttribute(new Float32Array(positions), 3));
  return geo;
}

export function buildFloor(): Group {
  const group = new Group();
  for (const stroke of FLOOR_STROKES) {
    const steps = stroke.d.includes("A") ? 72 : 2;
    const pts = samplePath(stroke.d, steps);
    const mesh = new Mesh(
      ribbon(pts, stroke.w),
      new MeshBasicMaterial({
        color: 0xffffff,
        transparent: true,
        opacity: stroke.a,
        depthWrite: false,
        side: DoubleSide,
      }),
    );
    mesh.renderOrder = 0;
    group.add(mesh);
  }
  return group;
}

export function disposeGroup(root: Object3D) {
  root.traverse((obj) => {
    const mesh = obj as Mesh;
    if (mesh.geometry) mesh.geometry.dispose();
    const material = mesh.material as MeshBasicMaterial | ShaderMaterial | MeshBasicMaterial[] | undefined;
    if (Array.isArray(material)) material.forEach((m) => m.dispose());
    else material?.dispose();
  });
}
