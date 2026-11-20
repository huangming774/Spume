import { useState } from "react";
import { useLocation } from "react-router-dom";
import { FiChevronDown, FiChevronUp, FiMusic, FiPause, FiPlay, FiSkipForward } from "react-icons/fi";
import { cosAsset } from "../lib/cosAsset.js";
import { usePlaylistAudio } from "../lib/usePlaylistAudio.js";

const FALLBACK_SRC = `${cosAsset(
  "AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/main",
)}/audio/loop.mp3`;

function fmt(sec) {
  if (!Number.isFinite(sec) || sec <= 0) return "0:00";
  const m = Math.floor(sec / 60);
  const s = Math.floor(sec % 60);
  return `${m}:${String(s).padStart(2, "0")}`;
}

const MiniPlayer = () => {
  const { pathname } = useLocation();
  const { isPlaying, toggle, next, seek, trackLabel, trackName, artist, mode, current, duration } =
    usePlaylistAudio(FALLBACK_SRC);
  const [collapsed, setCollapsed] = useState(false);

  // 后台界面不显示播放器。
  if (pathname.startsWith("/app")) return null;

  const title = trackName || (mode === "fallback" ? "站点背景音乐" : "网易云歌单");
  const subtitle = artist || (mode === "playlist" ? trackLabel : "loop");

  if (collapsed) {
    return (
      <button
        type="button"
        onClick={() => setCollapsed(false)}
        aria-label="展开音乐播放器"
        title={trackLabel || "背景音乐"}
        className="fixed bottom-5 right-5 z-40 flex h-12 w-12 items-center justify-center rounded-full border border-white/40 bg-white/70 text-[#b76e79] shadow-[0_10px_30px_rgba(183,110,121,0.25)] backdrop-blur-md transition hover:scale-105"
      >
        <FiMusic aria-hidden className={isPlaying ? "animate-pulse" : ""} />
      </button>
    );
  }

  return (
    <div className="fixed bottom-5 right-5 z-40 w-[290px] rounded-2xl border border-white/50 bg-white/75 p-4 shadow-[0_16px_44px_rgba(36,19,34,0.18)] backdrop-blur-xl">
      <div className="flex items-center gap-3">
        <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-[#ffd9e0] text-[#b76e79]">
          <FiMusic aria-hidden className={isPlaying ? "animate-pulse" : ""} />
        </span>
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-bold text-[#241322]">{title}</p>
          <p className="truncate text-xs text-[#8a6a70]">{subtitle}</p>
        </div>
        <button
          type="button"
          onClick={() => setCollapsed(true)}
          aria-label="收起播放器"
          className="text-[#8a6a70] transition hover:text-[#b76e79]"
        >
          <FiChevronDown aria-hidden />
        </button>
      </div>

      <input
        type="range"
        min={0}
        max={duration || 0}
        step={0.5}
        value={Math.min(current, duration || 0)}
        onChange={(e) => seek(Number(e.target.value))}
        aria-label="播放进度"
        className="mt-3 w-full accent-[#b76e79]"
      />
      <div className="mt-1 flex justify-between font-general text-[10px] uppercase tracking-[0.2em] text-[#8a6a70]">
        <span>{fmt(current)}</span>
        <span>{fmt(duration)}</span>
      </div>

      <div className="mt-2 flex items-center justify-center gap-4">
        <button
          type="button"
          onClick={toggle}
          aria-label={isPlaying ? "暂停" : "播放"}
          className="flex h-11 w-11 items-center justify-center rounded-full bg-[#241322] text-[#ffe7ef] transition hover:scale-105"
        >
          {isPlaying ? <FiPause aria-hidden /> : <FiPlay aria-hidden className="ml-0.5" />}
        </button>
        {mode === "playlist" && (
          <button
            type="button"
            onClick={next}
            aria-label="下一首"
            className="flex h-9 w-9 items-center justify-center rounded-full border border-[#e8cfc0] text-[#5f4b52] transition hover:bg-[#ffd9e0]"
          >
            <FiSkipForward aria-hidden />
          </button>
        )}
        <button
          type="button"
          onClick={() => setCollapsed(true)}
          aria-label="收起"
          className="flex h-9 w-9 items-center justify-center rounded-full border border-[#e8cfc0] text-[#5f4b52] transition hover:bg-[#ffd9e0]"
        >
          <FiChevronUp aria-hidden className="rotate-180" />
        </button>
      </div>
    </div>
  );
};

export default MiniPlayer;
