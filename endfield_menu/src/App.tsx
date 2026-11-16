import { useEffect, useLayoutEffect, useRef, useState, type CSSProperties, type MouseEvent } from "react";
import { BACKDROP } from "./art/backdrop";
import { ArtSprite } from "./art/ArtSprite";
import { MenuEngine, type Phase } from "./scene/engine";
import { MenuLayers } from "./ui/menu-layers";
import { NetInfo } from "./ui/chrome";

const ENTER_MS = 980;
const QUICK_ENTER_MS = 180;

export function App({
  onPhase,
  quick = false,
  active = true,
}: {
  onPhase?: (phase: Phase) => void;
  /** Site overlay: show the disc on the first frame instead of fading in for a third of a second. */
  quick?: boolean;
  /** False while the overlay is preloaded but not on screen. */
  active?: boolean;
} = {}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const domRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<HTMLDivElement>(null);
  const camRef = useRef<HTMLDivElement>(null);
  const engineRef = useRef<MenuEngine | null>(null);
  const [phase, setPhase] = useState<Phase>("enter");
  const [ready, setReady] = useState(false);
  const [toast, setToast] = useState("");
  const [field, setField] = useState({ image: "none", edge: "#1d2425", w: 2560, h: 1720 });
  const phaseRef = useRef(phase);
  phaseRef.current = phase;
  const skipSync = useRef(true);
  const engineMissing = useRef(false);

  const beginClose = () => {
    const now = phaseRef.current;
    if (now !== "open" && now !== "enter") {
      if (now === "closed") setPhase("enter");
      return;
    }
    setPhase(engineMissing.current ? "closed" : "exit");
  };

  useEffect(() => {
    onPhase?.(phase);
  }, [onPhase, phase]);

  useEffect(() => {
    const source = BACKDROP;
    let cancel = false;
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = 720;
      canvas.height = Math.max(1, Math.round(720 * (source.height / source.width)));
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      ctx.fillStyle = source.edge;
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.filter = "blur(10px)";
      ctx.drawImage(img, -16, -16, canvas.width + 32, canvas.height + 32);
      if (cancel) return;
      setField({ image: `url("${canvas.toDataURL("image/jpeg", 0.84)}")`, edge: source.edge, w: source.width, h: source.height });
    };
    img.src = source.src;
    return () => { cancel = true; };
  }, []);

  useLayoutEffect(() => {
    const canvas = canvasRef.current;
    const dom = domRef.current;
    const view = viewRef.current;
    const cam = camRef.current;
    if (!canvas || !dom || !view || !cam) return;
    const engine = (() => {
      try {
        return new MenuEngine({ canvas, dom, view, cam }, () => {
          if (phaseRef.current === "exit") setPhase("closed");
        }, { quick });
      } catch (error) {
        console.error(error);
        return null;
      }
    })();
    if (!engine) {
      engineMissing.current = true;
      setReady(true);
      return;
    }
    engineRef.current = engine;
    engine.renderNow();
    setReady(true);
    return () => {
      engine.destroy();
      engineRef.current = null;
    };
  }, []);

  useEffect(() => {
    if (skipSync.current) {
      skipSync.current = false;
      return;
    }
    engineRef.current?.setPhase(phase);
  }, [phase]);

  useEffect(() => {
    if (phase !== "enter") return;
    const id = window.setTimeout(() => setPhase("open"), quick ? QUICK_ENTER_MS : ENTER_MS);
    return () => window.clearTimeout(id);
  }, [phase, quick]);

  useLayoutEffect(() => {
    const engine = engineRef.current;
    if (!engine) return;
    if (!active) {
      engine.setActive(false);
      return;
    }
    if (phaseRef.current === "closed") {
      engine.setPhase("enter");
      setPhase("enter");
    }
    engine.setActive(true);
  }, [active]);

  useEffect(() => {
    if (!toast) return;
    const id = window.setTimeout(() => setToast(""), 1400);
    return () => window.clearTimeout(id);
  }, [toast]);

  useEffect(() => {
    if (!active) return undefined;
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      beginClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [active]);

  const onClick = (e: MouseEvent) => {
    const el = (e.target as Element).closest("[data-action]");
    if (!el) return;
    const action = el.getAttribute("data-action") || "";
    if (action === "close") {
      beginClose();
      return;
    }
    const label = el.getAttribute("aria-label") || action;
    setToast(label);
  };

  return (
    <div
      className="app"
      data-phase={phase}
      data-ready={ready ? "1" : "0"}
      onClick={onClick}
      style={{ "--field-w": String(field.w), "--field-h": String(field.h) } as CSSProperties}
    >
      <ArtSprite />
      <div className="wallpaper" style={{ backgroundImage: field.image, backgroundColor: field.edge }} />
      <div
        className="dim"
        onClick={() => beginClose()}
      />
      <canvas ref={canvasRef} className="gl" />
      <div className="css-root" ref={domRef}>
        <div className="css-view" ref={viewRef}>
          <MenuLayers camRef={camRef} phase={phase} />
        </div>
      </div>
      <div className="grade" />
      <div className="flat"><NetInfo /></div>
      {toast ? <div className="toast">{toast}</div> : null}
      <button type="button" className="hint" onClick={() => setPhase("enter")}>MENU CLOSED · 点击重新打开</button>
    </div>
  );
}
