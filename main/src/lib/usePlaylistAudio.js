import { useEffect, useSyncExternalStore } from "react";

/**
 * 站点背景音乐（模块级单例）：后台配了网易云歌单就逐首轮播（不可播/加载失败
 * 自动跳过，列表播完循环）；没配置或解析失败回退站点自带 loop.mp3 单曲循环。
 *
 * audio 元素全局只创建一次，导航栏均衡器按钮和首页迷你播放器共用同一实例，
 * 通过 useSyncExternalStore 订阅同一份状态，避免两个 audio 互相打架。
 */

const listeners = new Set();
let snapshot = {
  isPlaying: false,
  trackLabel: "",
  mode: "fallback", // "playlist" | "fallback"
  current: 0,
  duration: 0,
  trackName: "",
  artist: "",
  playlistName: "",
  allTracks: [],
  trackId: "",
};
let audio = null;
let tracks = [];
let index = 0;
let skipCount = 0;
let loaded = false;
let fallbackSrc = "";

function emit(patch) {
  snapshot = { ...snapshot, ...patch };
  listeners.forEach((listener) => listener());
}

function subscribe(listener) {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function getSnapshot() {
  return snapshot;
}

function labelOf(track, playlistName) {
  if (!track) return playlistName || "";
  const name = [track.name, track.artist].filter(Boolean).join(" - ");
  return name || playlistName || "";
}

function ensureAudio() {
  if (audio || typeof window === "undefined") return audio;
  audio = new Audio();
  audio.preload = "auto";
  audio.addEventListener("ended", () => {
    skipCount = 0;
    if (snapshot.mode === "playlist") advance();
  });
  audio.addEventListener("error", () => {
    if (!snapshot.isPlaying) return;
    if (snapshot.mode === "playlist") advance();
    else emit({ isPlaying: false });
  });
  audio.addEventListener("playing", () => {
    skipCount = 0;
  });
  audio.addEventListener("timeupdate", () => {
    emit({ current: audio.currentTime || 0 });
  });
  audio.addEventListener("loadedmetadata", () => {
    emit({ duration: audio.duration || 0 });
  });
  return audio;
}

function applyTrack(i, { autoplay }) {
  const el = ensureAudio();
  if (!el) return;
  if (snapshot.mode === "playlist" && tracks.length) {
    const track = tracks[i % tracks.length];
    index = i % tracks.length;
    emit({
      trackLabel: labelOf(track, ""),
      trackName: track.name || "",
      artist: track.artist || "",
      trackId: String(track.id ?? ""),
      current: 0,
      duration: 0,
    });
    el.src = track.url;
  } else {
    emit({ trackLabel: "", trackName: "", artist: "", current: 0, duration: 0 });
    el.src = fallbackSrc;
  }
  el.loop = snapshot.mode === "fallback";
  el.load();
  if (autoplay) el.play().catch(() => emit({ isPlaying: false }));
}

function advance() {
  if (snapshot.mode !== "playlist" || !tracks.length) {
    const el = ensureAudio();
    if (el) {
      el.currentTime = 0;
      el.play().catch(() => emit({ isPlaying: false }));
    }
    return;
  }
  skipCount += 1;
  if (skipCount > tracks.length) {
    // 整轮全部失败：回退默认音乐，避免死循环打接口。
    tracks = [];
    emit({ mode: "fallback", trackLabel: "", trackName: "", artist: "" });
    applyTrack(0, { autoplay: snapshot.isPlaying });
    return;
  }
  applyTrack(index + 1, { autoplay: true });
}

async function loadConfig() {
  try {
    const res = await fetch("/api/client/music", { headers: { Accept: "application/json" } });
    const config = res.ok ? await res.json() : null;
    if (!config?.enabled) {
      tracks = [];
      emit({ mode: "fallback", trackLabel: "", trackName: "", artist: "" });
      applyTrack(0, { autoplay: snapshot.isPlaying });
      return;
    }
    const listRes = await fetch("/api/client/music/playlist", {
      headers: { Accept: "application/json" },
    });
    if (!listRes.ok) throw new Error("playlist unavailable");
    const data = await listRes.json();
    const all = data.tracks || [];
    const playable = all.filter((track) => track.playable && track.url);
    if (!playable.length) throw new Error("no playable tracks");
    tracks = playable;
    index = 0;
    skipCount = 0;
    emit({ mode: "playlist", playlistName: data.playlistName || "", allTracks: all });
    applyTrack(0, { autoplay: snapshot.isPlaying });
  } catch {
    tracks = [];
    emit({ mode: "fallback", trackLabel: "", trackName: "", artist: "", playlistName: "", allTracks: [] });
    applyTrack(0, { autoplay: snapshot.isPlaying });
  }
}

function ensureLoaded() {
  if (loaded || typeof window === "undefined") return;
  loaded = true;
  loadConfig();
  window.addEventListener("blog-music-config-updated", () => loadConfig());
}

function toggle() {
  const el = ensureAudio();
  if (!el) return;
  if (snapshot.isPlaying) {
    el.pause();
    emit({ isPlaying: false });
  } else {
    if (!el.src) applyTrack(index, { autoplay: false });
    el.play()
      .then(() => emit({ isPlaying: true }))
      .catch(() => emit({ isPlaying: false }));
  }
}

function next() {
  if (snapshot.mode !== "playlist") return;
  skipCount = 0;
  applyTrack(index + 1, { autoplay: snapshot.isPlaying });
}

function prev() {
  if (snapshot.mode !== "playlist") return;
  skipCount = 0;
  applyTrack(index - 1 + tracks.length, { autoplay: snapshot.isPlaying });
}

function seek(time) {
  const el = ensureAudio();
  if (!el || !Number.isFinite(time)) return;
  el.currentTime = Math.max(0, Math.min(time, el.duration || time));
  emit({ current: el.currentTime });
}

/** 点击歌单里某首可播歌曲立即播放。 */
function playById(id) {
  const i = tracks.findIndex((track) => String(track.id) === String(id));
  if (i < 0) return;
  skipCount = 0;
  emit({ isPlaying: true });
  applyTrack(i, { autoplay: true });
}

/**
 * 订阅共享播放器状态。fallbackSrc 为没配歌单时的默认循环音乐地址。
 * 返回 { isPlaying, toggle, next, prev, seek, trackLabel, trackName, artist,
 *         mode, current, duration }。
 */
export function usePlaylistAudio(src) {
  const synced = useSyncExternalStore(subscribe, getSnapshot);
  useEffect(() => {
    if (src) fallbackSrc = src;
    ensureLoaded();
  }, [src]);
  return {
    ...synced,
    toggle,
    next,
    prev,
    seek,
    playById,
  };
}
