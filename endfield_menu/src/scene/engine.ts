import {
  Group,
  Object3D,
  PerspectiveCamera,
  Scene,
  WebGLRenderer,
} from "three";
import {
  CAM_DIST,
  LAYER_Z,
  PLUS_MARKS,
  REST_PITCH,
  SHIFT_X,
  SHIFT_Y,
  STAGE_H,
  STAGE_W,
  YELLOW_RING,
  ZOOM,
} from "../data";
import { cameraCSSMatrix, cssNum, objectCSSMatrix, stageMatrixToGroup } from "./matrices";
import { buildDisc, buildFloor, disposeGroup, type DiscKit } from "./meshes";

export type Phase = "enter" | "open" | "exit" | "closed";

export const stageView = { scale: 1 };

const DEG = Math.PI / 180;
const MAX_DPR = 1.5;

type Binding = { el: HTMLElement; obj: Object3D; last: string };

type Pose = { s: number; tilt: number; lift: number; reveal: number; alpha: number; skel: number };

function clamp(v: number, a: number, b: number) {
  return Math.max(a, Math.min(b, v));
}

function restingEye() {
  const r = 1 * DEG;
  const p = -REST_PITCH * DEG;
  const x0 = -SHIFT_X;
  const y0 = -SHIFT_Y;
  const x1 = x0 * Math.cos(r) - y0 * Math.sin(r);
  const y1 = x0 * Math.sin(r) + y0 * Math.cos(r);
  return {
    x: 960 + x1,
    y: 540 + y1 * Math.cos(p) - CAM_DIST * Math.sin(p),
    z: y1 * Math.sin(p) + CAM_DIST * Math.cos(p),
  };
}

function poseAt(phase: Phase, t: number, reduced: boolean, quick: boolean): Pose {
  if (reduced || phase === "open") return { s: 1, tilt: 0, lift: 0, reveal: 1, alpha: 1, skel: 0 };
  if (phase === "closed") return { s: 0.02, tilt: 40, lift: 20, reveal: 0, alpha: 0, skel: 0 };
  if (quick && phase === "enter") {
    const u = clamp(t / 0.16, 0, 1);
    const e = 1 - (1 - u) * (1 - u);
    return { s: 0.97 + 0.03 * e, tilt: 4 * (1 - e), lift: 0, reveal: 1, alpha: 1, skel: 0 };
  }
  if (phase === "exit") {
    const u = clamp(t / 0.34, 0, 1);
    return {
      s: 1 - 0.88 * u * u,
      tilt: 50 * u,
      lift: 28 * u,
      reveal: 1 - clamp((t - 0.16) / 0.12, 0, 1),
      alpha: 1 - clamp((t - 0.12) / 0.16, 0, 1),
      skel: t > 0.12 && t < 0.3 ? 0.55 : 0,
    };
  }
  const su = clamp((t - 0.06) / 0.48, 0, 1);
  const se = 1 - Math.pow(1 - su, 3);
  const tu = clamp((t - 0.08) / 0.32, 0, 1);
  const te = tu * tu * (3 - 2 * tu);
  const skel = t < 0.07 ? 0 : t < 0.38 ? 1 : t < 0.55 ? 1 - (t - 0.38) / 0.17 : 0;
  return {
    s: su <= 0 ? 0.02 : 0.02 + 0.98 * se,
    tilt: 38 * (1 - te),
    lift: (1 - se) * 24,
    reveal: clamp((t - 0.34) / 0.18, 0, 1),
    alpha: clamp((t - 0.32) / 0.08, 0, 1),
    skel,
  };
}

export class MenuEngine {
  private readonly scene = new Scene();
  private readonly camera: PerspectiveCamera;
  private readonly renderer: WebGLRenderer;
  private readonly pitchG = new Group();
  private readonly yawG = new Group();
  private readonly disc: DiscKit;
  private readonly bindings: Binding[] = [];
  private readonly reduced: boolean;
  private readonly onExitDone: () => void;
  private readonly camEl: HTMLElement;
  private readonly domEl: HTMLElement;
  private readonly viewEl: HTMLElement;
  private phase: Phase = "enter";
  private readonly quick: boolean;
  private shown = true;
  private phaseStart = 0;
  private exitSent = false;
  private running = false;
  private raf = 0;
  private camStyle = "";
  private poseKey = "";
  private vw = 1;
  private vh = 1;
  private readonly spring = { pitch: REST_PITCH, yaw: 0, vp: 0, vy: 0, tp: REST_PITCH, ty: 0 };

  constructor(
    hosts: { canvas: HTMLCanvasElement; dom: HTMLElement; view: HTMLElement; cam: HTMLElement },
    onExitDone: () => void,
    options: { quick?: boolean } = {},
  ) {
    this.quick = options.quick ?? false;
    this.camEl = hosts.cam;
    this.domEl = hosts.dom;
    this.viewEl = hosts.view;
    this.onExitDone = onExitDone;
    this.reduced = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ?? false;
    this.camera = new PerspectiveCamera(40, 1, 2, 9000);
    this.camera.position.set(0, 0, CAM_DIST);
    this.renderer = new WebGLRenderer({
      canvas: hosts.canvas,
      alpha: true,
      antialias: true,
      powerPreference: "high-performance",
      stencil: false,
    });
    this.renderer.setClearColor(0x000000, 1);

    const shifted = new Group();
    shifted.position.set(SHIFT_X, -SHIFT_Y, 0);
    const rolled = new Group();
    rolled.rotation.z = 1 * DEG;
    this.scene.add(shifted);
    shifted.add(rolled);
    rolled.add(this.pitchG);
    this.pitchG.add(this.yawG);

    this.disc = buildDisc();
    this.yawG.add(this.disc.group);
    this.yawG.add(buildFloor());
    this.bindDom();
    this.phaseStart = performance.now();
    this.resize();
    window.addEventListener("resize", this.resize);
    document.addEventListener("visibilitychange", this.onVis);
    window.addEventListener("mousemove", this.onMove);
  }

  setPhase(phase: Phase) {
    this.phase = phase;
    this.phaseStart = performance.now();
    this.exitSent = false;
    if (phase === "closed") {
      this.stop();
      return;
    }
    this.step(performance.now());
    if (this.shown) this.kick();
  }

  /** Pause drawing while the site overlay is hidden, and resume on the next click. */
  setActive(on: boolean) {
    this.shown = on;
    if (!on) {
      this.stop();
      return;
    }
    if (this.phase !== "closed") this.kick();
  }

  /** Draw one frame immediately so the first paint is already aligned, then keep the clock running. */
  renderNow() {
    this.step(performance.now());
    if (this.phase !== "closed") this.kick();
  }

  destroy() {
    this.stop();
    window.removeEventListener("resize", this.resize);
    document.removeEventListener("visibilitychange", this.onVis);
    window.removeEventListener("mousemove", this.onMove);
    disposeGroup(this.scene);
    this.renderer.dispose();
  }

  private bindDom() {
    const eye = restingEye();
    this.camEl.querySelectorAll<HTMLElement>("[data-layer]").forEach((el) => {
      const z = LAYER_Z[el.dataset.layer || ""] ?? 0;
      const k = (eye.z - z) / eye.z;
      const ctx = (1 - k) * (eye.x - 960);
      const cty = (1 - k) * (eye.y - 540);
      const obj = new Object3D();
      obj.position.set(ctx, -cty, z);
      obj.scale.set(k, k, 1);
      this.yawG.add(obj);
      this.bindings.push({ el, obj, last: "" });
    });

    const well = this.camEl.querySelector<HTMLElement>('[data-anchor="well"]');
    if (well) {
      const obj = new Object3D();
      obj.position.set(0, 0, -130);
      this.disc.group.add(obj);
      this.bindings.push({ el: well, obj, last: "" });
    }

    const level = this.camEl.querySelector<HTMLElement>('[data-anchor="level"]');
    if (level) {
      const obj = new Object3D();
      obj.position.set(-335, -276.2, 48);
      this.disc.group.add(obj);
      this.bindings.push({ el: level, obj, last: "" });
    }

    this.camEl.querySelectorAll<HTMLElement>("[data-decor]").forEach((el) => {
      const mark = PLUS_MARKS[Number(el.dataset.decor)];
      if (!mark) return;
      const obj = new Object3D();
      obj.matrixAutoUpdate = false;
      stageMatrixToGroup(mark.m, obj.matrix);
      this.yawG.add(obj);
      this.bindings.push({ el, obj, last: "" });
    });

    const ring = this.camEl.querySelector<HTMLElement>('[data-anchor="ring"]');
    if (ring) {
      const obj = new Object3D();
      obj.matrixAutoUpdate = false;
      stageMatrixToGroup(YELLOW_RING.m, obj.matrix);
      this.yawG.add(obj);
      this.bindings.push({ el: ring, obj, last: "" });
    }
  }

  private readonly onMove = (e: MouseEvent) => {
    if (!this.shown || this.reduced || this.phase === "closed" || this.phase === "exit") return;
    const mx = (e.clientX / Math.max(1, window.innerWidth)) * 2 - 1;
    const my = (e.clientY / Math.max(1, window.innerHeight)) * 2 - 1;
    this.spring.tp = REST_PITCH + my * 2.5;
    this.spring.ty = -mx * 4;
    this.kick();
  };

  private readonly onVis = () => {
    if (document.hidden) this.stop();
    else if (this.shown && this.phase !== "closed") this.kick();
  };

  private readonly resize = () => {
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    if (vw < 2 || vh < 2) return;
    this.vw = vw;
    this.vh = vh;
    const fit = Math.min(vw / STAGE_W, vh / STAGE_H);
    stageView.scale = fit;
    document.documentElement.style.setProperty("--stage-scale", fit.toFixed(4));
    const visibleH = vh / (fit * ZOOM);
    this.camera.fov = (2 * Math.atan(visibleH / 2 / CAM_DIST) * 180) / Math.PI;
    this.camera.aspect = vw / vh;
    this.camera.updateProjectionMatrix();
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, MAX_DPR));
    this.renderer.setSize(vw, vh, false);
    this.domEl.style.width = vw + "px";
    this.domEl.style.height = vh + "px";
    this.viewEl.style.width = vw + "px";
    this.viewEl.style.height = vh + "px";
    this.camEl.style.width = vw + "px";
    this.camEl.style.height = vh + "px";
    this.camStyle = "";
    this.poseKey = "";
    for (const b of this.bindings) b.last = "";
    if (!this.running) this.step(performance.now());
  };

  private kick() {
    if (this.running) return;
    this.running = true;
    this.raf = requestAnimationFrame(this.loop);
  }

  private stop() {
    this.running = false;
    cancelAnimationFrame(this.raf);
  }

  private readonly loop = (now: number) => {
    this.step(now);
    if (this.running) this.raf = requestAnimationFrame(this.loop);
  };

  private step(now: number) {
    const t = (now - this.phaseStart) / 1000;
    const reduced = this.reduced;
    const pose = poseAt(this.phase, t, reduced, this.quick);

    if (!reduced && this.phase !== "closed") {
      const dx = this.spring.tp - this.spring.pitch;
      const dy = this.spring.ty - this.spring.yaw;
      this.spring.vp = (this.spring.vp + dx * 0.008) * 0.8;
      this.spring.vy = (this.spring.vy + dy * 0.008) * 0.8;
      const settled =
        Math.abs(dx) < 2e-3 && Math.abs(dy) < 2e-3 && Math.abs(this.spring.vp) < 6e-4 && Math.abs(this.spring.vy) < 6e-4;
      if (settled) {
        this.spring.pitch = this.spring.tp;
        this.spring.yaw = this.spring.ty;
        this.spring.vp = 0;
        this.spring.vy = 0;
      } else {
        this.spring.pitch += this.spring.vp;
        this.spring.yaw += this.spring.vy;
      }
    }
    this.pitchG.rotation.x = -this.spring.pitch * DEG;
    this.yawG.rotation.y = this.spring.yaw * DEG;

    this.disc.group.scale.setScalar(Math.max(pose.s, 0.001));
    this.disc.group.rotation.x = -pose.tilt * DEG;
    this.disc.group.position.y = pose.lift;
    this.disc.uniforms.uTime.value = now / 1000;
    this.disc.uniforms.uReveal.value = pose.reveal;
    this.disc.uniforms.uAlpha.value = pose.alpha;
    this.disc.skelMat.opacity = pose.skel * 0.9;
    this.disc.skel.visible = pose.skel > 0.02;

    if (!reduced && this.phase === "open") {
      const cyc = ((now / 1000) % 90) / 45;
      const u = cyc < 1 ? cyc : 2 - cyc;
      this.disc.flags.rotation.z = (-30 + 60 * u) * DEG;
    } else {
      this.disc.flags.rotation.z = 0;
    }

    this.scene.updateMatrixWorld();
    this.camera.updateMatrixWorld();
    this.renderer.render(this.scene, this.camera);

    const poseKey =
      pose.s.toFixed(3) + "/" + pose.tilt.toFixed(2) + "/" + pose.lift.toFixed(2) +
      "/" + this.spring.pitch.toFixed(3) + "/" + this.spring.yaw.toFixed(3);
    if (poseKey !== this.poseKey) {
      this.poseKey = poseKey;
      this.writeDom();
    }

    if (this.phase === "exit" && t > 0.52 && !this.exitSent) {
      this.exitSent = true;
      this.onExitDone();
    }
  }

  private writeDom() {
    const fovPx = this.camera.projectionMatrix.elements[5] * (this.vh / 2);
    const style =
      "perspective(" + cssNum(fovPx) + "px) translateZ(" + cssNum(fovPx) + "px)" +
      cameraCSSMatrix(this.camera.matrixWorldInverse.elements) +
      "translate(" + cssNum(this.vw / 2) + "px," + cssNum(this.vh / 2) + "px)";
    if (style !== this.camStyle) {
      this.camEl.style.transform = style;
      this.camStyle = style;
    }
    for (const b of this.bindings) {
      const next = objectCSSMatrix(b.obj.matrixWorld.elements);
      if (next !== b.last) {
        b.el.style.transform = next;
        b.last = next;
      }
    }
  }
}
