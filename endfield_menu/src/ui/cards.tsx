import { memo } from "react";
import { CARD_CONTOURS } from "../art/contours";
import { CARD_ANGLES, CARD_H, CARD_SHAPES, CARDS, DISC, R_EDGE } from "../data";
import { cardOutline, platePath } from "../cardShape";
import { GlyphText } from "./glyphs";

const R_CARD = R_EDGE + 16;
const ANCHOR_X = 350;

function cardBox(index: number) {
  const a = (CARD_ANGLES[index] * Math.PI) / 180;
  const ax = DISC.cx - R_CARD * Math.cos(a);
  const ay = DISC.cy - R_CARD * Math.sin(a);
  return { left: ax - ANCHOR_X, top: ay - CARD_H / 2 };
}

function contourD(ln: number[]) {
  let d = `M${ln[2]} ${ln[3]}`;
  for (let k = 4; k < ln.length; k += 2) d += `L${ln[k]} ${ln[k + 1]}`;
  return d;
}

function Contours({ index }: { index: number }) {
  const lines = CARD_CONTOURS[index];
  if (!lines?.length) return null;
  return (
    <g className="contour">
      {lines.map((ln, i) => (
        <path key={i} d={contourD(ln)} strokeWidth={ln[0]} strokeOpacity={ln[1]} />
      ))}
    </g>
  );
}

export const Cards = memo(function Cards() {
  return (
    <div className="cards">
      {CARDS.map((card, i) => {
        const box = cardBox(i);
        const outline = cardOutline(i, 0);
        const ring = cardOutline(i, 4);
        const shadow = CARD_SHAPES[i].shadow;
        return (
          <div key={card.action} className="card" style={{ left: box.left, top: box.top, zIndex: 5 + i, animationDelay: `${0.32 + i * 0.06}s` }}>
            <svg className="card-shadow" viewBox="0 0 440 172" style={{ left: shadow[0], top: shadow[1] }} aria-hidden="true">
              <path d={outline} />
            </svg>
            <button type="button" className="card-hit" data-action={card.action} aria-label={card.label} style={{ clipPath: `path("${outline}")` }}>
              <span className="card-face" style={{ clipPath: `path("${outline}")` }}>
                <svg className="card-plate" viewBox="0 0 440 172" aria-hidden="true">
                  <path className="plate" d={platePath(i)} />
                  <Contours index={i} />
                  <path className="hl" d={outline} />
                </svg>
                <svg className="card-art" viewBox="0 0 440 172" aria-hidden="true">
                  {card.noted ? <use href={`#${card.art}-ring`} fill="#FDEF00" /> : null}
                  <use href={`#${card.art}`} />
                  {card.sheen ? <use href={`#${card.art}-sheen`} fill="#575757" /> : null}
                  <GlyphText text={card.label} g={card.text} />
                </svg>
              </span>
              <svg className="card-ring" viewBox="0 0 440 172" aria-hidden="true">
                <path d={ring} />
              </svg>
            </button>
          </div>
        );
      })}
    </div>
  );
});

export const Cables = memo(function Cables() {
  const bundles = CARD_ANGLES.map((_, i) => {
    const a = (CARD_ANGLES[i] * Math.PI) / 180;
    const ux = Math.cos(a);
    const uy = Math.sin(a);
    const tx = -Math.sin(a);
    const ty = Math.cos(a);
    const px = (r: number, o: number) => DISC.cx - r * ux + o * tx;
    const py = (r: number, o: number) => DISC.cy - r * uy + o * ty;
    const rOut = R_CARD - 20;
    const rTop = DISC.bezelR + 80;
    const rFoot = DISC.bezelR + 34;
    const rEnd = DISC.bezelR + 10;
    const pt = (x: number, y: number) => `${x.toFixed(1)} ${y.toFixed(1)}`;
    const paths = [-6.9, -2.3, 2.2, 6.8].map(
      (o) =>
        `M${pt(px(rOut, o), py(rOut, o))} L${pt(px(rTop, o), py(rTop, o))} L${pt(px(rFoot, o) + 3, py(rFoot, o) + 13)} L${pt(px(rEnd, o) + 3, py(rEnd, o) + 13)}`,
    );
    return { paths, x1: px(rOut, 0), y1: py(rOut, 0), x2: px(rEnd, 0) + 3, y2: py(rEnd, 0) + 13 };
  });

  return (
    <svg className="cables" viewBox="0 0 1920 1080" aria-hidden="true">
      <defs>
        {bundles.map((b, i) => (
          <linearGradient key={i} id={`cable${i}`} gradientUnits="userSpaceOnUse" x1={b.x1} y1={b.y1} x2={b.x2} y2={b.y2}>
            <stop offset="0" stopColor="rgba(255,255,255,0.06)" />
            <stop offset="0.22" stopColor="rgba(255,255,255,0.5)" />
            <stop offset="0.75" stopColor="rgba(255,255,255,0.5)" />
            <stop offset="1" stopColor="rgba(255,255,255,0.04)" />
          </linearGradient>
        ))}
      </defs>
      {bundles.map((b, i) => (
        <g key={i} style={{ animationDelay: "0.27s" }}>
          {b.paths.map((d) => (
            <path key={d} d={d} fill="none" stroke={`url(#cable${i})`} strokeWidth="2.7" strokeLinejoin="round" strokeLinecap="round" />
          ))}
        </g>
      ))}
    </svg>
  );
});

export const NewsBelt = memo(function NewsBelt() {
  const d = [
    "M1488.64 193.04L1436.18 230.51A48.55 48.55 0 0 0 1429.75 236.01L1387.56 279",
    "M1491.95 197.68L1439.5 235.15A42.85 42.85 0 0 0 1433.82 240L1389.65 285",
    "M1495.27 202.32L1442.81 239.79A37.15 37.15 0 0 0 1437.89 244L1393.71 289",
    "M1498.58 206.96L1446.12 244.43A31.45 31.45 0 0 0 1441.96 247.99L1397.77 293",
  ];
  return (
    <svg className="belt" viewBox="0 0 1920 1080" aria-hidden="true">
      <defs>
        <linearGradient id="beltGrad" gradientUnits="userSpaceOnUse" x1="0" y1="200" x2="0" y2="293">
          <stop offset="0.237" stopColor="#EBA118" stopOpacity="0" />
          <stop offset="0.366" stopColor="#EBA118" />
          <stop offset="0.538" stopColor="#EBA118" stopOpacity="0.97" />
          <stop offset="0.796" stopColor="#EBA118" stopOpacity="0.85" />
          <stop offset="1" stopColor="#EBA118" stopOpacity="0.55" />
        </linearGradient>
      </defs>
      {d.map((path) => (
        <path key={path} d={path} fill="none" stroke="url(#beltGrad)" strokeWidth="2.6" />
      ))}
    </svg>
  );
});
