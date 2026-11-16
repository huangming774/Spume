import { useLayoutEffect, useRef } from "react";
import { DISC, R_EDGE, TILE_ARC_Y, TILE_FIRST, TILE_ROWS, TILE_STEP, TILE_VIEW, TILE_VISIBLE, TILES, ZOOM } from "../data";
import { stageView } from "../scene/engine";
import { GlyphText } from "./glyphs";

const PX_PER_DEG = (TILE_ARC_Y * Math.PI) / 180;
const MAX_SCROLL = Math.max(0, TILE_ROWS - TILE_VISIBLE) * TILE_STEP;
const MAX_OVERSCROLL = 354 / PX_PER_DEG;

function rowPosition(scroll: number, row: number) {
  const a = ((TILE_FIRST - row * TILE_STEP + scroll) * Math.PI) / 180;
  return {
    x: DISC.cx + R_EDGE * Math.cos(a) - TILE_VIEW.x,
    y: DISC.cy - 50 - TILE_ARC_Y * a - 22 - TILE_VIEW.y,
  };
}

function rubber(raw: number) {
  if (raw < 0) return -MAX_OVERSCROLL * (1 - Math.exp((0.5 * raw) / MAX_OVERSCROLL));
  if (raw > MAX_SCROLL) return MAX_SCROLL + MAX_OVERSCROLL * (1 - Math.exp((-0.5 * (raw - MAX_SCROLL)) / MAX_OVERSCROLL));
  return raw;
}

export function Tiles() {
  const vpRef = useRef<HTMLDivElement>(null);
  const rowsRef = useRef<Array<HTMLDivElement | null>>([]);
  const upRef = useRef<HTMLButtonElement>(null);
  const downRef = useRef<HTMLButtonElement>(null);
  const api = useRef<(dir: number) => void>(() => {});

  useLayoutEffect(() => {
    const vp = vpRef.current;
    if (!vp) return;
    let scroll = 0;
    let target = 0;
    let easeRaf = 0;
    let springRaf = 0;
    let coastRaf = 0;
    let springTimer = 0;
    let dragging = false;
    let dragged = false;
    let dragStartY = 0;
    let dragStartScroll = 0;
    let lastY = 0;
    let lastT = 0;
    let velocity = 0;

    const layout = () => {
      for (let i = 0; i < TILE_ROWS; i += 1) {
        const el = rowsRef.current[i];
        if (!el) continue;
        const p = rowPosition(scroll, i);
        el.style.transform = `translate3d(${p.x.toFixed(1)}px, ${p.y.toFixed(1)}px, 0) rotate(-3deg)`;
      }
      upRef.current?.classList.toggle("is-on", scroll > 0.3);
      downRef.current?.classList.toggle("is-on", scroll < MAX_SCROLL - 0.3);
    };

    const stop = () => {
      cancelAnimationFrame(easeRaf);
      cancelAnimationFrame(springRaf);
      cancelAnimationFrame(coastRaf);
      clearTimeout(springTimer);
    };

    const overshoot = () => (scroll < 0 ? scroll : scroll > MAX_SCROLL ? scroll - MAX_SCROLL : 0);

    const springBack = () => {
      cancelAnimationFrame(springRaf);
      const step = () => {
        const rest = scroll < 0 ? 0 : MAX_SCROLL;
        const off = scroll - rest;
        if (Math.abs(off) < 0.02) {
          scroll = target = rest;
          layout();
          return;
        }
        scroll = rest + off * 0.8;
        layout();
        springRaf = requestAnimationFrame(step);
      };
      step();
    };

    const easeTo = () => {
      cancelAnimationFrame(easeRaf);
      const step = () => {
        const d = target - scroll;
        if (Math.abs(d) < 0.02) {
          scroll = target;
          layout();
          if (overshoot() !== 0) springTimer = window.setTimeout(springBack, 60);
          return;
        }
        scroll += d * 0.18;
        layout();
        easeRaf = requestAnimationFrame(step);
      };
      step();
    };

    const coast = (v: number) => {
      const step = () => {
        if (Math.abs(v) < 2e-3) {
          target = scroll;
          return;
        }
        scroll += v;
        if (scroll < 0 || scroll > MAX_SCROLL) {
          scroll = target = Math.max(-MAX_OVERSCROLL, Math.min(MAX_SCROLL + MAX_OVERSCROLL, scroll));
          springBack();
          return;
        }
        v *= 0.92;
        layout();
        coastRaf = requestAnimationFrame(step);
      };
      step();
    };

    api.current = (dir: number) => {
      stop();
      target = Math.max(0, Math.min(MAX_SCROLL, scroll + dir * TILE_STEP));
      easeTo();
    };

    const onWheel = (e: WheelEvent) => {
      e.preventDefault();
      stop();
      let d = e.deltaY * 0.062;
      const over = target < 0 ? target : target > MAX_SCROLL ? target - MAX_SCROLL : 0;
      if (over !== 0) d *= Math.max(0.04, Math.pow(1 - Math.abs(over) / MAX_OVERSCROLL, 2));
      target = Math.max(-MAX_OVERSCROLL, Math.min(MAX_SCROLL + MAX_OVERSCROLL, target + d));
      easeTo();
    };

    const onDown = (e: PointerEvent) => {
      stop();
      dragging = true;
      dragged = false;
      dragStartY = lastY = e.clientY;
      dragStartScroll = target = scroll;
      lastT = performance.now();
      velocity = 0;
      vp.classList.add("is-drag");
      vp.setPointerCapture(e.pointerId);
    };

    const onMove = (e: PointerEvent) => {
      if (!dragging) return;
      const dy = e.clientY - dragStartY;
      if (Math.abs(dy) > 6) dragged = true;
      const now = performance.now();
      velocity = (e.clientY - lastY) / Math.max(1, now - lastT);
      lastY = e.clientY;
      lastT = now;
      const px = PX_PER_DEG * Math.max(0.2, stageView.scale) * ZOOM;
      scroll = target = rubber(dragStartScroll - dy / px);
      layout();
    };

    const endDrag = (inertia: boolean) => {
      if (!dragging) return;
      dragging = false;
      vp.classList.remove("is-drag");
      window.setTimeout(() => {
        dragged = false;
      }, 0);
      if (overshoot() !== 0) springBack();
      else if (inertia) coast((-velocity * 16) / (PX_PER_DEG * Math.max(0.2, stageView.scale) * ZOOM));
      else target = scroll;
    };

    const onClick = (e: MouseEvent) => {
      if (!dragged) return;
      e.preventDefault();
      e.stopPropagation();
    };
    const onUp = () => endDrag(true);
    const onCancel = () => endDrag(false);

    layout();
    vp.addEventListener("wheel", onWheel, { passive: false });
    vp.addEventListener("pointerdown", onDown);
    vp.addEventListener("pointermove", onMove);
    vp.addEventListener("pointerup", onUp);
    vp.addEventListener("pointercancel", onCancel);
    vp.addEventListener("click", onClick, true);
    return () => {
      stop();
      vp.removeEventListener("wheel", onWheel);
      vp.removeEventListener("pointerdown", onDown);
      vp.removeEventListener("pointermove", onMove);
      vp.removeEventListener("pointerup", onUp);
      vp.removeEventListener("pointercancel", onCancel);
      vp.removeEventListener("click", onClick, true);
    };
  }, []);

  return (
    <div className="tiles" ref={vpRef}>
      <button type="button" ref={upRef} className="nav nav-up" aria-label="向上" onClick={(e) => { e.stopPropagation(); api.current(-1); }}>
        <i /><span />
      </button>
      {Array.from({ length: TILE_ROWS }, (_, r) => (
        <div
          key={r}
          className="row"
          ref={(el) => { rowsRef.current[r] = el; }}
          style={{ zIndex: 40 + r, animationDelay: `${0.28 + Math.min(r, 5) * 0.04}s` }}
        >
          {TILES.slice(r * 2, r * 2 + 2).map((tile, col) => (
            <button key={tile.id} type="button" className="tile" data-action={tile.id} aria-label={tile.label} style={{ left: col === 0 ? 0 : 128 }}>
              <i className="tile-shadow" />
              <span className="tile-face">
                <svg viewBox="0 0 107 104" aria-hidden="true">
                  <use href={`#${tile.art}`} />
                  <GlyphText text={tile.label} g={tile.text} />
                </svg>
              </span>
              {tile.dot ? <i className="dot" /> : null}
            </button>
          ))}
        </div>
      ))}
      <button type="button" ref={downRef} className="nav nav-down is-on" aria-label="向下" onClick={(e) => { e.stopPropagation(); api.current(1); }}>
        <span /><i />
      </button>
    </div>
  );
}
