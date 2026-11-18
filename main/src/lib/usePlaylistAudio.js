import { useCallback, useEffect, useRef, useState } from "react";

/**
 * 导航栏背景音乐：后台配了网易云歌单就逐首轮播（不可播/加载失败自动跳过，
 * 列表播完循环）；没配置或解析失败回退站点自带 loop.mp3 单曲循环。
 * 返回 { audioRef, isPlaying, toggle, trackLabel }，audio 元素由调用方渲染。
 */
export function usePlaylistAudio(fallbackSrc) {
  const audioRef = useRef(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [trackLabel, setTrackLabel] = useState("");
  const [tracks, setTracks] = useState([]);
  const [mode, setMode] = useState("fallback"); // "playlist" | "fallback"
  const indexRef = useRef(0);
  // 一轮里连续跳过的次数；超过可播总数说明全挂了，回退默认音乐。
  const skipCountRef = useRef(0);
  const tracksRef = useRef([]);
  const modeRef = useRef("fallback");
  tracksRef.current = tracks;
  modeRef.current = mode;

  const loadConfig = useCallback(async () => {
    try {
      const res = await fetch("/api/client/music", { headers: { Accept: "application/json" } });
      const config = res.ok ? await res.json() : null;
      if (!config?.enabled) {
        setTracks([]);
        setMode("fallback");
        setTrackLabel("");
        return;
      }
      const listRes = await fetch("/api/client/music/playlist", {
        headers: { Accept: "application/json" },
      });
      if (!listRes.ok) throw new Error("playlist unavailable");
      const data = await listRes.json();
      const playable = (data.tracks || []).filter((track) => track.playable && track.url);
      if (!playable.length) throw new Error("no playable tracks");
      setTracks(playable);
      setMode("playlist");
      indexRef.current = 0;
      skipCountRef.current = 0;
      setTrackLabel(labelOf(playable[0], data.playlistName));
    } catch {
      setTracks([]);
      setMode("fallback");
      setTrackLabel("");
    }
  }, []);

  useEffect(() => {
    loadConfig();
    const onConfigUpdated = () => loadConfig();
    window.addEventListener("blog-music-config-updated", onConfigUpdated);
    return () => window.removeEventListener("blog-music-config-updated", onConfigUpdated);
  }, [loadConfig]);

  const applyTrack = useCallback(
    (index, { autoplay }) => {
      const audio = audioRef.current;
      if (!audio) return;
      const list = tracksRef.current;
      if (modeRef.current === "playlist" && list.length) {
        const track = list[index % list.length];
        indexRef.current = index % list.length;
        setTrackLabel(labelOf(track, ""));
        audio.src = track.url;
      } else {
        audio.src = fallbackSrc;
      }
      audio.load();
      if (autoplay) {
        audio.play().catch(() => setIsPlaying(false));
      }
    },
    [fallbackSrc],
  );

  // 播放状态 → audio 播放/暂停（沿用原实现：点击按钮在用户手势里触发）。
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;
    if (isPlaying) {
      if (!audio.src) applyTrack(indexRef.current, { autoplay: false });
      audio.play().catch(() => setIsPlaying(false));
    } else {
      audio.pause();
    }
  }, [isPlaying, applyTrack]);

  // 模式/列表变化时装载当前曲目；正在播则无缝换源继续。
  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;
    const wasPlaying = isPlaying;
    applyTrack(indexRef.current, { autoplay: wasPlaying });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mode, tracks]);

  const advance = useCallback(() => {
    const list = tracksRef.current;
    if (modeRef.current !== "playlist" || !list.length) {
      // 兜底单曲循环
      const audio = audioRef.current;
      if (audio) {
        audio.currentTime = 0;
        audio.play().catch(() => setIsPlaying(false));
      }
      return;
    }
    skipCountRef.current += 1;
    if (skipCountRef.current > list.length) {
      // 整轮全部失败：回退默认音乐，避免死循环打接口。
      setMode("fallback");
      setTracks([]);
      setTrackLabel("");
      return;
    }
    applyTrack(indexRef.current + 1, { autoplay: true });
  }, [applyTrack]);

  const onEnded = useCallback(() => {
    skipCountRef.current = 0;
    if (modeRef.current === "playlist") {
      advance();
    }
    // fallback 模式由 audio loop 属性自己循环
  }, [advance]);

  const onError = useCallback(() => {
    if (!isPlaying) return;
    if (modeRef.current === "playlist") {
      advance();
    } else {
      setIsPlaying(false);
    }
  }, [advance, isPlaying]);

  const onPlaying = useCallback(() => {
    skipCountRef.current = 0;
  }, []);

  const toggle = useCallback(() => setIsPlaying((prev) => !prev), []);

  return { audioRef, isPlaying, toggle, trackLabel, onEnded, onError, onPlaying, mode };
}

function labelOf(track, playlistName) {
  if (!track) return playlistName || "";
  const name = [track.name, track.artist].filter(Boolean).join(" - ");
  return name || playlistName || "";
}
