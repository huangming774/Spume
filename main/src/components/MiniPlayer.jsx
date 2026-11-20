import { useState } from "react";
import { useLocation } from "react-router-dom";
import {
  FiChevronDown,
  FiList,
  FiMusic,
  FiPause,
  FiPlay,
  FiSkipBack,
  FiSkipForward,
} from "react-icons/fi";
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

/** 播放中的小均衡器动画（三条跳动竖线）。 */
const EqBars = ({ active }) => (
  <span className="flex h-3.5 items-end gap-[2px]" aria-hidden="true">
    {[0, 1, 2].map((i) => (
      <span
        key={i}
        className={`w-[3px] rounded-full bg-current ${active ? "mini-eq-bar" : "h-[3px]"}`}
        style={active ? { animationDelay: `${i * 0.18}s` } : undefined}
      />
    ))}
  </span>
);

const MiniPlayer = () => {
  const { pathname } = useLocation();
  const {
    isPlaying,
    toggle,
    next,
    prev,
    seek,
    playById,
    trackLabel,
    trackName,
    artist,
    mode,
    current,
    duration,
    playlistName,
    allTracks,
    trackId,
  } = usePlaylistAudio(FALLBACK_SRC);
  const [collapsed, setCollapsed] = useState(true);
  const [showList, setShowList] = useState(false);
  const playableCount = allTracks.filter((t) => t.playable).length;

  // 后台界面不显示播放器。
  if (pathname.startsWith("/app")) return null;

  const title = trackName || (mode === "fallback" ? "站点背景音乐" : "网易云歌单");
  const subtitle = artist || (mode === "playlist" ? trackLabel : "loop · mp3");

  // 收起态：右上角小胶囊，播放时带跳动均衡器。
  if (collapsed) {
    return (
      <button
        type="button"
        onClick={() => setCollapsed(false)}
        aria-label="展开音乐播放器"
        title={trackLabel || "背景音乐"}
        className="mini-player-root group fixed right-4 top-20 z-40 flex items-center gap-2 rounded-full border border-white/60 bg-white/70 py-2 pl-3 pr-4 text-[#b76e79] shadow-[0_10px_30px_rgba(183,110,121,0.22)] backdrop-blur-xl transition hover:-translate-y-0.5 hover:bg-white/85 sm:right-6 sm:top-24"
      >
        <span className="flex h-7 w-7 items-center justify-center rounded-full bg-[linear-gradient(135deg,#ffd9e0,#ffe9c9)] text-[#b76e79]">
          <FiMusic aria-hidden className="h-3.5 w-3.5" />
        </span>
        <EqBars active={isPlaying} />
        <span className="font-general text-[10px] uppercase tracking-[0.28em] text-[#8a6a70] transition group-hover:text-[#b76e79]">
          {isPlaying ? "Playing" : "Music"}
        </span>
      </button>
    );
  }

  return (
    <div className="mini-player-root fixed right-4 top-20 z-40 w-[300px] overflow-hidden rounded-3xl border border-white/60 bg-white/75 shadow-[0_24px_60px_rgba(36,19,34,0.22)] backdrop-blur-2xl sm:right-6 sm:top-24">
      {/* 顶部玫瑰金晕光 */}
      <div
        aria-hidden="true"
        className="pointer-events-none absolute -top-16 left-1/2 h-32 w-[130%] -translate-x-1/2 rounded-full bg-[radial-gradient(closest-side,rgba(255,143,171,0.28),transparent)] blur-xl"
      />

      <div className="relative p-4">
        <div className="flex items-start justify-between">
          <p className="font-general text-[10px] uppercase tracking-[0.34em] text-[#b76e79]">
            Now Playing
          </p>
          <button
            type="button"
            onClick={() => setCollapsed(true)}
            aria-label="收起播放器"
            className="-mr-1 -mt-1 flex h-7 w-7 items-center justify-center rounded-full text-[#8a6a70] transition hover:bg-[#ffd9e0]/60 hover:text-[#b76e79]"
          >
            <FiChevronDown aria-hidden className="h-4 w-4" />
          </button>
        </div>

        <div className="mt-3 flex items-center gap-3">
          {/* 旋转唱片 */}
          <span
            className={`relative flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-[conic-gradient(from_0deg,#241322,#5f4b52,#241322,#8a6a70,#241322)] shadow-[0_6px_18px_rgba(36,19,34,0.35)] ${
              isPlaying ? "animate-[spin_5s_linear_infinite]" : ""
            }`}
            aria-hidden="true"
          >
            <span className="flex h-5 w-5 items-center justify-center rounded-full bg-[linear-gradient(135deg,#ff8fab,#ffd9e0)]">
              <span className="h-1.5 w-1.5 rounded-full bg-white/90" />
            </span>
          </span>
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-bold text-[#241322]" title={title}>
              {title}
            </p>
            <p className="mt-0.5 truncate text-xs text-[#8a6a70]" title={subtitle}>
              {subtitle}
            </p>
          </div>
        </div>

        <input
          type="range"
          min={0}
          max={duration || 0}
          step={0.5}
          value={Math.min(current, duration || 0)}
          onChange={(e) => seek(Number(e.target.value))}
          aria-label="播放进度"
          className="mini-player-range mt-4 w-full"
        />
        <div className="mt-1 flex justify-between font-general text-[10px] uppercase tracking-[0.22em] text-[#8a6a70]">
          <span>{fmt(current)}</span>
          <span>{fmt(duration)}</span>
        </div>

        <div className="mt-3 flex items-center justify-center gap-3">
          {mode === "playlist" && (
            <button
              type="button"
              onClick={prev}
              aria-label="上一首"
              className="flex h-9 w-9 items-center justify-center rounded-full border border-[#f2d8c8] bg-white/60 text-[#5f4b52] transition hover:-translate-y-0.5 hover:bg-[#ffd9e0]"
            >
              <FiSkipBack aria-hidden className="h-4 w-4" />
            </button>
          )}
          <button
            type="button"
            onClick={toggle}
            aria-label={isPlaying ? "暂停" : "播放"}
            className="flex h-12 w-12 items-center justify-center rounded-full bg-[#241322] text-[#ffe7ef] shadow-[0_10px_26px_rgba(36,19,34,0.35)] transition hover:scale-105 hover:bg-[#3a2130]"
          >
            {isPlaying ? (
              <FiPause aria-hidden className="h-5 w-5" />
            ) : (
              <FiPlay aria-hidden className="ml-0.5 h-5 w-5" />
            )}
          </button>
          {mode === "playlist" && (
            <button
              type="button"
              onClick={next}
              aria-label="下一首"
              className="flex h-9 w-9 items-center justify-center rounded-full border border-[#f2d8c8] bg-white/60 text-[#5f4b52] transition hover:-translate-y-0.5 hover:bg-[#ffd9e0]"
            >
              <FiSkipForward aria-hidden className="h-4 w-4" />
            </button>
          )}
          {mode === "playlist" && (
            <button
              type="button"
              onClick={() => setShowList((v) => !v)}
              aria-label="查看歌单"
              title={`歌单 · ${playableCount}/${allTracks.length} 首可播`}
              className={`flex h-9 w-9 items-center justify-center rounded-full border transition hover:-translate-y-0.5 ${
                showList
                  ? "border-[#b76e79] bg-[#ffd9e0] text-[#b76e79]"
                  : "border-[#f2d8c8] bg-white/60 text-[#5f4b52] hover:bg-[#ffd9e0]"
              }`}
            >
              <FiList aria-hidden className="h-4 w-4" />
            </button>
          )}
        </div>

        {showList && mode === "playlist" && (
          <div className="mt-3 border-t border-[#f2d8c8] pt-3">
            <div className="flex items-baseline justify-between">
              <p className="truncate text-xs font-bold text-[#241322]">
                {playlistName || "歌单"}
              </p>
              <span className="font-general text-[10px] uppercase tracking-[0.2em] text-[#8a6a70]">
                {playableCount}/{allTracks.length} 可播
              </span>
            </div>
            {playableCount < allTracks.length && (
              <p className="mt-1 text-[10px] leading-4 text-[#8a6a70]">
                灰色为 VIP/版权受限歌曲，网易云不提供外链，自动跳过。
              </p>
            )}
            <ul className="mini-track-list mt-2 max-h-56 space-y-1 overflow-y-auto pr-1">
              {allTracks.map((t) => {
                const active = String(t.id) === trackId;
                return (
                  <li key={t.id}>
                    <button
                      type="button"
                      disabled={!t.playable}
                      onClick={() => playById(t.id)}
                      title={t.playable ? "播放这首" : t.reason || "暂无外链"}
                      className={`flex w-full items-center gap-2 rounded-xl px-2.5 py-1.5 text-left transition ${
                        active
                          ? "bg-[#ffd9e0] text-[#b76e79]"
                          : t.playable
                            ? "text-[#5f4b52] hover:bg-[#ffd9e0]/50"
                            : "cursor-not-allowed text-[#c9b6ba] opacity-60"
                      }`}
                    >
                      <span className="w-4 shrink-0 text-center">
                        {active && isPlaying ? (
                          <EqBars active />
                        ) : (
                          <FiMusic aria-hidden className="mx-auto h-3 w-3" />
                        )}
                      </span>
                      <span className="min-w-0 flex-1">
                        <span className="block truncate text-xs font-semibold">{t.name}</span>
                        <span className="block truncate text-[10px] opacity-80">{t.artist}</span>
                      </span>
                      {!t.playable && (
                        <span className="shrink-0 rounded-full bg-[#f2d8c8]/70 px-1.5 py-0.5 text-[9px] text-[#8a6a70]">
                          {t.reason?.includes("VIP") ? "VIP" : t.reason?.includes("付费") ? "付费" : "版权"}
                        </span>
                      )}
                    </button>
                  </li>
                );
              })}
            </ul>
          </div>
        )}
      </div>
    </div>
  );
};

export default MiniPlayer;
