import { useEffect, useRef, useState } from "react";
import { fetchServerInfo } from "../services/serverInfoApi";

// ───────────────── 数字缓动 ─────────────────
function useCountUp(target, duration = 900) {
  const [val, setVal] = useState(0);
  const fromRef = useRef(0);
  useEffect(() => {
    if (typeof target !== "number" || Number.isNaN(target)) return;
    const start = performance.now();
    const from = fromRef.current;
    const delta = target - from;
    let raf;
    const tick = (now) => {
      const t = Math.min(1, (now - start) / duration);
      const eased = 1 - Math.pow(1 - t, 3);
      setVal(from + delta * eased);
      if (t < 1) raf = requestAnimationFrame(tick);
      else fromRef.current = target;
    };
    raf = requestAnimationFrame(tick);
    return () => raf && cancelAnimationFrame(raf);
  }, [target, duration]);
  return val;
}

// ───────────────── 圆形 HUD（奶油主题） ─────────────────
const CircleHUD = ({
  percent,
  centerPrimary,
  centerSecondary,
  label,
  hint,
  accent = "#74C0FC",
  glow = "#A5D8FF",
  size = 220,
  showTicks = true,
  spin = true,
}) => {
  const p = Math.max(0, Math.min(100, percent || 0));
  const animated = useCountUp(p, 1200);

  const cx = size / 2;
  const cy = size / 2;
  const outerR = size / 2 - 4;
  const trackR = outerR - 16;
  const innerR = trackR - 14;
  const stroke = 6;
  const circ = 2 * Math.PI * trackR;
  const offset = circ - (animated / 100) * circ;

  const ticks = [];
  if (showTicks) {
    const total = 60;
    for (let i = 0; i < total; i++) {
      const angle = (i / total) * 360 - 90;
      const rad = (angle * Math.PI) / 180;
      const isMajor = i % 5 === 0;
      const inner = outerR - (isMajor ? 8 : 4);
      ticks.push(
        <line
          key={i}
          x1={cx + Math.cos(rad) * outerR}
          y1={cy + Math.sin(rad) * outerR}
          x2={cx + Math.cos(rad) * inner}
          y2={cy + Math.sin(rad) * inner}
          stroke={accent}
          strokeOpacity={isMajor ? 0.7 : 0.32}
          strokeWidth={isMajor ? 1.3 : 0.9}
        />
      );
    }
  }

  const arcId = `arc-${accent.replace("#", "")}-${size}`;

  return (
    <div
      className="server-hud-card relative aspect-square w-full overflow-hidden rounded-full"
      style={{
        // 奶油白底 + 内侧柔光（替代深蓝）
        background:
          `radial-gradient(circle at 30% 25%, #FFFFFF 0%, #FFFBF5 55%, #FFF6E4 100%)`,
        boxShadow:
          `0 0 0 1px ${accent}55, 0 14px 32px ${glow}55, inset 0 0 0 6px #FFFFFF, inset 0 0 32px ${accent}22`,
      }}
    >
      {/* 旋转光弧（柔色，叠在奶油底上） */}
      <div
        className="pointer-events-none absolute inset-0 rounded-full"
        style={{
          background: `conic-gradient(from 0deg, transparent 0deg, ${accent}66 22deg, transparent 44deg, transparent 360deg)`,
          animation: spin ? "hud-spin 7s linear infinite" : "none",
          mixBlendMode: "multiply",
          opacity: 0.55,
        }}
        aria-hidden
      />

      {/* 边缘内辉光 */}
      <div
        className="pointer-events-none absolute inset-2 rounded-full"
        style={{
          boxShadow: `inset 0 0 14px ${accent}44`,
        }}
        aria-hidden
      />

      <svg
        className="absolute inset-0 h-full w-full"
        viewBox={`0 0 ${size} ${size}`}
      >
        <defs>
          <linearGradient id={arcId} x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor={accent} stopOpacity="1" />
            <stop offset="100%" stopColor={glow} stopOpacity="0.55" />
          </linearGradient>
          <radialGradient id={`${arcId}-glow`} cx="50%" cy="50%" r="50%">
            <stop offset="55%" stopColor="transparent" />
            <stop offset="100%" stopColor={accent} stopOpacity="0.10" />
          </radialGradient>
        </defs>

        {ticks}

        <circle
          cx={cx}
          cy={cy}
          r={trackR}
          stroke={accent}
          strokeOpacity="0.2"
          strokeWidth={stroke}
          fill="none"
        />
        <circle
          cx={cx}
          cy={cy}
          r={trackR}
          stroke={`url(#${arcId})`}
          strokeWidth={stroke}
          fill="none"
          strokeLinecap="round"
          strokeDasharray={circ}
          strokeDashoffset={offset}
          transform={`rotate(-90 ${cx} ${cy})`}
          style={{
            filter: `drop-shadow(0 0 4px ${accent}aa)`,
            transition: "stroke-dashoffset 0.4s linear",
          }}
        />
        <circle
          cx={cx}
          cy={cy}
          r={innerR}
          stroke={accent}
          strokeOpacity="0.5"
          strokeWidth="1"
          strokeDasharray="2 4"
          fill="none"
        />
        <circle cx={cx} cy={cy} r={innerR - 6} fill={`url(#${arcId}-glow)`} />
      </svg>

      <div className="absolute inset-0 flex flex-col items-center justify-center px-4 text-center">
        <div
          className="server-hud-label font-zentry text-[10px] font-semibold uppercase tracking-[0.3em]"
          style={{ color: accent }}
        >
          {label}
        </div>
        <div className="server-hud-primary mt-1 font-zentry text-2xl font-black text-[#2B2B2B] sm:text-3xl md:text-4xl">
          {centerPrimary ?? `${animated.toFixed(0)}%`}
        </div>
        {centerSecondary && (
          <div className="server-hud-secondary mt-1 text-[10px] font-medium tracking-wider text-[#6B7280] md:text-[11px]">
            {centerSecondary}
          </div>
        )}
        {hint && (
          <div
            className="server-hud-hint absolute bottom-5 left-1/2 -translate-x-1/2 text-[9px] uppercase tracking-[0.25em] md:text-[10px]"
            style={{ color: `${accent}cc` }}
          >
            {hint}
          </div>
        )}
      </div>
    </div>
  );
};

// ───────────────── Mock 数据（仅本地预览） ─────────────────
const MOCK_INFO = {
  vendor: "UCloud 香港",
  region: "亚太 · 香港",
  os: "Linux",
  arch: "amd64",
  goVersion: "go1.22.3",
  cpuCores: 4,
  goroutines: 87,
  memory: { usedMB: 1280, totalMB: 4096, usedRatio: 0.3125, source: "system" },
  cpuPercent: 32.4,
  uptime: "12 天 4 小时",
  uptimeSecs: 12 * 86400 + 4 * 3600,
  status: "online",
  serverTime: "2026-06-22T06:30:00Z",
};

const isMockMode = () => {
  if (typeof window === "undefined") return false;
  const sp = new URLSearchParams(window.location.search);
  return sp.get("mock") === "1";
};

// ───────────────── 工具 ─────────────────
const formatMB = (mb) => {
  if (!Number.isFinite(mb)) return "—";
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${Math.round(mb)} MB`;
};

// ───────────────── 主面板 ─────────────────
const ServerInfoPanel = () => {
  const [info, setInfo] = useState(null);
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    if (isMockMode()) {
      setInfo(MOCK_INFO);
      setLoading(false);
      const timer = setInterval(() => {
        if (cancelled) return;
        setInfo((prev) => ({
          ...(prev || MOCK_INFO),
          cpuPercent: +(20 + Math.random() * 50).toFixed(1),
          goroutines: 70 + Math.floor(Math.random() * 40),
          serverTime: new Date().toISOString(),
        }));
      }, 2500);
      return () => {
        cancelled = true;
        clearInterval(timer);
      };
    }
    const load = async () => {
      try {
        const data = await fetchServerInfo();
        if (!cancelled) {
          setInfo(data);
          setError("");
        }
      } catch (e) {
        if (!cancelled) setError(e.message || "加载失败");
      } finally {
        if (!cancelled) setLoading(false);
      }
    };
    load();
    const timer = setInterval(load, 10000);
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, []);

  const cpuPercent = info?.cpuPercent ?? 0;
  const memPercent = info?.memory ? Math.round(info.memory.usedRatio * 100) : 0;
  const memText = info?.memory ? formatMB(info.memory.usedMB) : "—";
  const memTotalText = info?.memory ? `/ ${formatMB(info.memory.totalMB)}` : "";

  const uptimeForRing = (() => {
    if (!info?.uptimeSecs) return 0;
    const day = 86400;
    const pct = (info.uptimeSecs / (30 * day)) * 100;
    return Math.min(100, pct);
  })();

  return (
    <section
      className="server-info-panel relative overflow-hidden rounded-[24px] border border-[#F2E6C9] p-4 sm:p-5 md:p-7"
      style={{
        // 奶油白卡背景（跟 AI KPI 卡同源）
        background:
          "linear-gradient(135deg, #FFFFFF 0%, #FFFBF0 50%, #FFF6E4 100%)",
        boxShadow:
          "0 18px 42px rgba(255,143,171,0.10), inset 0 0 0 1px rgba(255,212,59,0.18)",
      }}
    >
      {/* 装饰光晕 */}
      <div
        className="pointer-events-none absolute -right-16 -top-16 h-56 w-56 rounded-full bg-[#FFD43B]/25 blur-3xl"
        aria-hidden
      />
      <div
        className="pointer-events-none absolute -bottom-20 -left-16 h-56 w-56 rounded-full bg-[#A5D8FF]/25 blur-3xl"
        aria-hidden
      />
      <div
        className="pointer-events-none absolute right-10 bottom-10 h-40 w-40 rounded-full bg-[#FF8FAB]/15 blur-3xl"
        aria-hidden
      />

      {/* 顶部标题栏 */}
      <div className="relative flex flex-wrap items-center gap-3">
        <span className="relative flex h-2.5 w-2.5">
          <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-[#51CF66] opacity-70" />
          <span className="relative inline-flex h-2.5 w-2.5 rounded-full bg-[#37B24D] shadow-[0_0_8px_rgba(81,207,102,0.7)]" />
        </span>
        <h2 className="text-lg font-bold tracking-wide text-[#2B2B2B] md:text-xl">
          <span className="mr-1 text-[#FFD43B]">◆</span>
          服务器状态
        </h2>
        <span className="text-[10px] font-semibold uppercase tracking-[0.3em] text-[#6B7280]">
          live telemetry
        </span>
        <div className="ml-auto flex flex-wrap items-center gap-2">
          {info?.vendor && (
            <span className="rounded-full border border-[#FFD43B]/50 bg-[#FFF8E7] px-3 py-0.5 text-[10px] font-bold tracking-wider text-[#A07C00]">
              {info.vendor}
            </span>
          )}
          {info?.region && (
            <span className="rounded-full border border-[#A5D8FF]/60 bg-[#EAF6FF] px-3 py-0.5 text-[10px] font-bold tracking-wider text-[#1971C2]">
              {info.region}
            </span>
          )}
        </div>
      </div>

      {loading && !info && (
        <p className="relative mt-6 text-sm text-[#6B7280]">▸ 正在连接遥测通道…</p>
      )}
      {error && (
        <p className="relative mt-6 rounded-2xl border border-[#FFC9C9] bg-white px-4 py-3 text-sm text-[#C92A2A]">
          ▲ {error}
        </p>
      )}

      {info && (
        <div className="server-info-grid relative mt-5 grid grid-cols-2 gap-3 sm:mt-6 sm:gap-4 lg:grid-cols-4 lg:gap-5">
          <CircleHUD
            label="CPU"
            percent={cpuPercent}
            centerPrimary={`${Number(cpuPercent).toFixed(1)}%`}
            centerSecondary={`${info.cpuCores ?? "?"} 核 · ${info.goroutines ?? "?"} 协程`}
            hint="processor"
            accent="#74C0FC"
            glow="#A5D8FF"
          />
          <CircleHUD
            label="MEM"
            percent={memPercent}
            centerPrimary={memText}
            centerSecondary={
              memTotalText || (info.memory?.source === "process" ? "process" : "")
            }
            hint={info.memory?.source === "process" ? "dev mode" : "system"}
            accent="#FF8FAB"
            glow="#FFC9C9"
          />
          <CircleHUD
            label="UPTIME"
            percent={uptimeForRing}
            centerPrimary={info.uptime || "—"}
            centerSecondary="自上次启动"
            hint="continuous"
            accent="#E8B500"
            glow="#FFD43B"
            showTicks
          />
          <CircleHUD
            label="RUNTIME"
            percent={100}
            centerPrimary={info.os || "—"}
            centerSecondary={info.goVersion || ""}
            hint={info.arch || ""}
            accent="#B197FC"
            glow="#D0BFFF"
            spin={false}
          />
        </div>
      )}

      {info && (
        <div className="relative mt-5 flex flex-wrap items-center justify-between gap-2 rounded-2xl border border-[#F2E6C9] bg-white/70 px-4 py-2 text-[10px] font-medium uppercase tracking-[0.25em] text-[#6B7280]">
          <span>◇ status: <span className="text-[#2B8A3E]">{info.status === "online" ? "ONLINE" : info.status}</span></span>
          <span className="hidden md:inline">◇ serverTime: {info.serverTime?.replace("T", " ").replace("Z", " UTC")}</span>
          <span>◇ telemetry: secure</span>
        </div>
      )}

      <style>{`
        @keyframes hud-spin {
          from { transform: rotate(0deg); }
          to   { transform: rotate(360deg); }
        }
      `}</style>
    </section>
  );
};

export default ServerInfoPanel;
