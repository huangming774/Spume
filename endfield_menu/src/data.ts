/** Stage layout measured from the fan recreation (1920×1080). */

export const STAGE_W = 1920;
export const STAGE_H = 1080;
export const CAM_DIST = 1600;
export const ZOOM = 1.1;
export const REST_PITCH = 20;
export const SHIFT_X = 29;
export const SHIFT_Y = -40;

export const DISC = { cx: 960, cy: 540, bezelR: 492 * 0.96 };
export const R_EDGE = DISC.bezelR + 110;

export type Glyph = {
  size: number;
  matrix?: number[];
  x: number[];
  y: number[];
};

export type CardDef = {
  action: string;
  label: string;
  art: string;
  noted?: boolean;
  sheen?: boolean;
  text: Glyph;
};

export const CARDS: CardDef[] = [
  { action: "operators", label: "干员", art: "art-card-operators", text: { size: 22.62, matrix: [1.0392, -0.0417, -0.0168, 1.0015], x: [203.11, 226.39], y: [145.07, 145.3] } },
  { action: "events", label: "活动", art: "art-card-events", text: { size: 24.07, matrix: [0.9983, -0.0576, -0.0123, 1.0024], x: [216.09, 240.97], y: [148.8, 149.39] } },
  { action: "headhunt", label: "寻访", art: "art-card-headhunt", noted: true, sheen: true, text: { size: 24.2, matrix: [1.0678, -0.0186, 0.0015, 1.0001], x: [202.57, 224.93], y: [147.04, 146.12] } },
  { action: "shop", label: "采购中心", art: "art-card-shop", text: { size: 24.52, matrix: [0.9796, -0.0274, -0.0261, 1.0011], x: [193.01, 218.59, 243.56, 268.83], y: [132.78, 132.55, 133.08, 132.58] } },
];

export const CARD_W = 440;
export const CARD_H = 172;
export const CARD_ANGLES = [28.02, 10.52, -6.22, -21.56];

/** top/bottom: y = a + b·x; left/right: x = a + b·y + c·y² */
export type CardShape = {
  top: number[];
  bottom: number[];
  left: number[];
  right: number[];
  plate: { top: number[]; left: number[] };
  shadow: [number, number];
};

export const CARD_SHAPES: CardShape[] = [
  { top: [22.19, -0.03542], bottom: [168.99, -0.03362], left: [173.3, -0.54768, 6.5769e-4], right: [399.48, -0.61494, 3.6623e-4], plate: { top: [38.65, -0.0352], left: [196.74, -0.56595, 6.3797e-4] }, shadow: [14, 20.3] },
  { top: [17.81, -0.03285], bottom: [170.39, -0.03319], left: [156.26, -0.40245, 1.4242e-3], right: [362.1, -0.28868, 6.3431e-4], plate: { top: [31.58, -0.03529], left: [176.6, -0.41875, 1.2589e-3] }, shadow: [15.9, 16.2] },
  { top: [16.02, -0.03221], bottom: [175.66, -0.0329], left: [125.98, -0.15145, 1.4506e-3], right: [330.5, 0.01888, 1.0545e-3], plate: { top: [26.28, -0.0346], left: [143.06, -0.1786, 1.3852e-3] }, shadow: [16.5, 12] },
  { top: [34.97, -0.02944], bottom: [153.99, -0.03242], left: [99.86, 0.28886, 3.757e-4], right: [320.36, 0.33845, 8.8469e-4], plate: { top: [41.71, -0.03279], left: [113.23, 0.2459, 5.216e-4] }, shadow: [15, 8.6] },
];

export type TileDef = { id: string; label: string; art: string; dot?: boolean; text: Glyph };

export const TILES: TileDef[] = [
  { id: "handbook", label: "行动手册", art: "art-tile-handbook", text: { size: 22.1, matrix: [1.003, -0.0018, -0.0043, 1], x: [9.07, 31.66, 54.16, 76.9], y: [91.43, 91.41, 91.7, 91] } },
  { id: "pass", label: "通行证", art: "art-tile-pass", text: { size: 21.39, matrix: [1.05, 0.0055, -0.0152, 0.9999], x: [20.26, 41.56, 62.87], y: [90.33, 90.65, 90.1] } },
  { id: "region", label: "地区建设", art: "art-tile-region", dot: true, text: { size: 21.74, matrix: [1.0199, -0.016, 0.0057, 1], x: [7.79, 30.72, 52.49, 74.55], y: [91.43, 91.55, 92.07, 92.15] } },
  { id: "friends", label: "好友", art: "art-tile-friends", text: { size: 22.14, matrix: [1.0298, 0.018, -0.0105, 1], x: [31.45, 53.15], y: [90.31, 89.27] } },
  { id: "crafting", label: "装备加工", art: "art-tile-crafting", text: { size: 22.41, matrix: [0.9999, 0.0122, -0.0052, 1], x: [9.8, 31.56, 54.46, 76.52], y: [86.85, 86.99, 86.96, 86.63] } },
  { id: "squad", label: "编队", art: "art-tile-squad", text: { size: 21.64, matrix: [1.02, 0, -0.003, 1], x: [30.63, 52.91], y: [90.69, 90.88] } },
  { id: "wiki", label: "百科", art: "art-tile-wiki", text: { size: 21.61, matrix: [1.03, 0.0036, -0.0005, 1], x: [30.2, 52.17], y: [90.64, 90.81] } },
  { id: "vault", label: "贵重品库", art: "art-tile-vault", text: { size: 20.57, matrix: [1.05, 0.0055, -0.0052, 1], x: [10.27, 31.11, 51.64, 72.17], y: [90.97, 90.33, 89.97, 89.86] } },
  { id: "exchange", label: "塔晶集换", art: "art-tile-exchange", text: { size: 21.95, matrix: [1.0129, -0.0159, 0.0057, 1], x: [7.84, 30.54, 52.63, 74.72], y: [88.31, 88.5, 88.84, 89.79] } },
  { id: "archive", label: "档案库", art: "art-tile-archive", text: { size: 21.84, matrix: [1.023, 0.0054, 0.0018, 1], x: [19.7, 41.94, 63.99], y: [91.42, 91.77, 91.36] } },
  { id: "backpack", label: "背包", art: "art-tile-backpack", text: { size: 22.29, matrix: [1.02, -0.0053, -0.0148, 1.0001], x: [31.29, 53.18], y: [91.47, 91.12] } },
  { id: "glory", label: "光荣之路", art: "art-tile-glory", text: { size: 21.07, matrix: [1.04, -0.0073, 0, 1], x: [8.6, 30.35, 51.45, 73.14], y: [88, 88.51, 87.62, 87.92] } },
  { id: "etching", label: "基质蚀刻", art: "art-tile-etching", text: { size: 21.28, matrix: [1.03, 0.0054, 0.0048, 1], x: [8.26, 30.04, 51.47, 73.61], y: [90.51, 90.69, 89.84, 90.35] } },
  { id: "missions", label: "任务", art: "art-tile-missions", text: { size: 22.43, matrix: [0.992, 0.0035, -0.0075, 1], x: [31.9, 54.45], y: [90.89, 90.88] } },
  { id: "map", label: "地图", art: "art-tile-map", text: { size: 21.59, matrix: [1.0124, -0.0336, 0.0202, 0.9999], x: [29.15, 51.86], y: [92.42, 92.77] } },
  { id: "baker", label: "Baker", art: "art-tile-baker", text: { size: 20.95, matrix: [1.0368, -0.0181, 0.0045, 1.0001], x: [22.03, 36.68, 48.19, 60.19, 71.82], y: [92, 92.09, 92.1, 92.4, 92.42] } },
  { id: "survey", label: "问卷中心", art: "art-tile-survey", text: { size: 20.87, matrix: [1, 0.007, 0.006, 1.0001], x: [10.27, 31.76, 53.21, 74.72], y: [90.61, 90.75, 90.31, 89.73] } },
  { id: "tools", label: "游戏工具", art: "art-tile-tools", text: { size: 22.26, matrix: [1.01, -0.0071, -0.003, 1], x: [8.58, 30.87, 53.3, 75.65], y: [91.25, 91.28, 91.55, 91.65] } },
];

export const TILE_ROWS = Math.ceil(TILES.length / 2);
export const TILE_VISIBLE = 5;
export const TILE_STEP = 15.5;
export const TILE_FIRST = 22.52;
export const TILE_ARC_Y = 490;
export const TILE_VIEW = { x: 1290, y: 200 };

export const SLIDES = [
  { caption: "版本前瞻 // 敬请期待", c1: "#7e3d49", c2: "#23191f" },
  { caption: "限时活动 · 进行中", c1: "#3d4f63", c2: "#161c24" },
  { caption: "新功能上线预告", c1: "#4f5a3d", c2: "#1a2016" },
  { caption: "系统维护预告 · 02:00", c1: "#5a4a2f", c2: "#221c12" },
  { caption: "终末地补给 · 限时供应", c1: "#5c5420", c2: "#1d1a0b" },
];

export type DiscButton = {
  action: string;
  label: string;
  art: string;
  decal: string;
  top: number;
  side: number;
  band: number;
  path: string;
  text: Glyph;
};

export const DISC_BUTTONS: DiscButton[] = [
  {
    action: "plan", label: "工业计划", art: "art-disc-plan", decal: "art-disc-square",
    top: 857.93, side: 959.23, band: 927.83,
    path: "M 664.87 857.93 L 868.36 857.93 A 395.42 395.42 0 0 0 955.75 870.68 A 3.6 3.6 0 0 1 959.23 874.28 L 959.23 957.79 A 5.6 5.6 0 0 1 953.55 963.39 C 853.66 961.98 739.35 925.45 664.87 857.93 Z",
    text: { size: 24.03, matrix: [1.0099, -0.0141, 0.024, 0.9998], x: [812.04, 836.44, 860.76, 884.82], y: [924.66, 923.98, 924.08, 922.98] },
  },
  {
    action: "brief", label: "工业简报", art: "art-disc-brief", decal: "art-disc-decal",
    top: 859.43, side: 978.37, band: 929.42,
    path: "M 1273.78 859.43 L 1069.24 859.43 A 395.42 395.42 0 0 1 981.85 872.18 A 3.6 3.6 0 0 0 978.37 875.78 L 978.37 959.29 A 5.6 5.6 0 0 0 984.05 964.89 C 1087.69 963.43 1200.22 925.6 1273.78 859.43 Z",
    text: { size: 23.65, matrix: [1.0198, -0.0196, 0.0322, 0.9996], x: [949.53, 973.67, 998.29, 1022.58], y: [927.15, 926.73, 926.26, 926.12] },
  },
];

export const FLOOR_STROKES = [
  { d: "M1711.9 184.1A811.8 811.8 0 0 1 1561.1 1109.2", w: 2.5, a: 0.135 },
  { d: "M1470.9 -101.6A1323.8 1323.8 0 0 1 1889.6 517.4", w: 3.9, a: 0.148 },
  { d: "M226.2 175.6A798.5 798.5 0 0 1 490.9 -120.7", w: 2, a: 0.125 },
  { d: "M50 813.9A753.3 753.3 0 0 1 190.8 229", w: 2, a: 0.122 },
  { d: "M1607.8 909.4L1755.7 1011.5", w: 3.6, a: 0.155 },
];

function m3(s: string): number[] {
  return s.slice(s.indexOf("(") + 1, s.lastIndexOf(")")).split(",").map(Number);
}

export type PlusMark = { m: number[]; g: number[]; a: number; wh: number; wv: number };

const PLUS_SRC: Array<[string, number[], number, number, number]> = [
  ["matrix3d(0.9997,-0.0256,-0.0021,0,0.0025,0.1787,-0.9839,0,0.0256,0.9836,0.1787,0,552.58,321.64,400.18,1)", [2.21, 9.01, 2.46, 10.46], 1, 1.74, 1.46],
  ["matrix3d(0.9997,-0.0256,-0.0021,0,0.0025,0.1787,-0.9839,0,0.0256,0.9836,0.1787,0,555.62,403.45,-2.28,1)", [2.35, 8.88, 2.59, 10.84], 0.5, 1.68, 1.41],
  ["matrix3d(0.9997,-0.0256,-0.0021,0,0.0025,0.1787,-0.9839,0,0.0256,0.9836,0.1787,0,567.41,752.3,75.35,1)", [2.55, 8.67, 2.87, 10.72], 0.47, 1.49, 1.29],
  ["matrix3d(0.9997,-0.0256,-0.0021,0,0.0025,0.1787,-0.9839,0,0.0256,0.9836,0.1787,0,1066.56,577.79,510,1)", [3.14, 10.76, 2.53, 9.75], 0.81, 1.47, 1.56],
  ["matrix3d(0.9997,-0.0256,-0.0021,0,0.0025,0.1787,-0.9839,0,0.0256,0.9836,0.1787,0,1363.12,368.46,23.39,1)", [2.33, 8.89, 2.7, 10.09], 0.51, 1.76, 1.51],
  ["matrix3d(0.9997,-0.0256,-0.0021,0,0.0025,0.1787,-0.9839,0,0.0256,0.9836,0.1787,0,1359.33,638.07,507.77,1)", [2.31, 8.91, 2.51, 10.04], 1, 1.53, 1.16],
  ["matrix3d(0.9997,-0.0256,-0.0021,0,0.0025,0.1787,-0.9839,0,0.0256,0.9836,0.1787,0,1366.15,723.62,114.34,1)", [2.43, 8.79, 2.86, 10.56], 0.51, 1.61, 1.25],
  ["matrix3d(0.9997,-0.0256,-0.0021,0,0.0025,0.1787,-0.9839,0,0.0256,0.9836,0.1787,0,1362.66,950.41,580,1)", [2.91, 8.31, 3.42, 9], 1, 1.61, 1.41],
  ["matrix3d(0.9997,-0.0256,-0.0021,0,0.0025,0.1787,-0.9839,0,0.0256,0.9836,0.1787,0,1383.69,1033.13,151.61,1)", [2.39, 8.84, 2.27, 11.51], 0.49, 1.37, 1.07],
];

export const PLUS_MARKS: PlusMark[] = PLUS_SRC.map(([s, g, a, wh, wv]) => ({ m: m3(s), g, a, wh, wv }));

export const YELLOW_RING = {
  m: m3("matrix3d(-0.4919,-0.8564,-0.1568,0,0.3794,-0.373,0.8467,0,-0.7836,0.357,0.5084,0,1706.99,905.47,-37.7,1)"),
  r: 7.4,
};

/** Heights above the floor plane. DOM layers use these; the disc itself is a WebGL group. */
export const LAYER_Z: Record<string, number> = {
  deck: 0,
  btns: 26,
  tiles: 28,
  cards: 34,
  news: 72,
  top: 144,
};
