import clsx from "clsx";
import { useCallback, useEffect, useRef, useState } from "react";
import { LuSearch } from "react-icons/lu";
import { useNavigate } from "react-router-dom";
import { searchSite } from "../services/searchApi.js";
import { isSearchable, normalizeHits } from "./siteSearchState.js";

const DEBOUNCE_MS = 300;

/**
 * 导航栏搜索：图标点开输入框，输入即搜（防抖 300ms），
 * 命中在下方浮层列出，点击直达文章或建站日志。
 */
const SiteSearch = ({ triggerClassName }) => {
  const navigate = useNavigate();
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState([]);
  const [status, setStatus] = useState("idle"); // idle | loading | done | unconfigured | error
  const [error, setError] = useState("");
  const [activeIndex, setActiveIndex] = useState(-1);

  const rootRef = useRef(null);
  const inputRef = useRef(null);
  const triggerRef = useRef(null);

  const close = useCallback(() => {
    setOpen(false);
    setActiveIndex(-1);
    requestAnimationFrame(() => {
      triggerRef.current?.focus({ preventScroll: true });
    });
  }, []);

  useEffect(() => {
    if (open) {
      inputRef.current?.focus({ preventScroll: true });
    }
  }, [open]);

  // 点到面板外面就收起。
  useEffect(() => {
    if (!open) return undefined;
    const onPointerDown = (event) => {
      if (!rootRef.current?.contains(event.target)) {
        setOpen(false);
        setActiveIndex(-1);
      }
    };
    document.addEventListener("mousedown", onPointerDown);
    return () => document.removeEventListener("mousedown", onPointerDown);
  }, [open]);

  useEffect(() => {
    if (!open) return undefined;
    const onKeyDown = (event) => {
      if (event.key === "Escape") close();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [open, close]);

  // 输入即搜：防抖 + 用 AbortController 丢弃过期响应。
  useEffect(() => {
    if (!open || !isSearchable(query)) {
      setHits([]);
      setStatus("idle");
      setError("");
      setActiveIndex(-1);
      return undefined;
    }
    const controller = new AbortController();
    const timer = window.setTimeout(async () => {
      setStatus("loading");
      setError("");
      try {
        const data = await searchSite(query.trim(), { signal: controller.signal });
        setHits(normalizeHits(data.items));
        setActiveIndex(-1);
        setStatus(data.configured === false ? "unconfigured" : "done");
      } catch (requestError) {
        if (controller.signal.aborted) return;
        setHits([]);
        setActiveIndex(-1);
        setError(requestError.message || "搜索失败，请稍后再试");
        setStatus("error");
      }
    }, DEBOUNCE_MS);
    return () => {
      window.clearTimeout(timer);
      controller.abort();
    };
  }, [open, query]);

  const goTo = (url) => {
    close();
    navigate(url);
  };

  const onSubmit = (event) => {
    event.preventDefault();
    const target = hits[activeIndex >= 0 ? activeIndex : 0];
    if (target) goTo(target.url);
  };

  const onInputKeyDown = (event) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setActiveIndex((index) => (hits.length ? (index + 1) % hits.length : -1));
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      setActiveIndex((index) =>
        hits.length ? (index - 1 + hits.length) % hits.length : -1
      );
    }
  };

  return (
    <div ref={rootRef} className="relative shrink-0">
      <button
        ref={triggerRef}
        type="button"
        className={clsx(
          triggerClassName,
          "flex h-9 w-9 items-center justify-center rounded-full",
          open && "bg-white/10"
        )}
        aria-label="站内搜索"
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <LuSearch aria-hidden="true" />
      </button>

      {open && (
        <form
          role="search"
          onSubmit={onSubmit}
          className="absolute right-0 top-full z-50 mt-2 w-[min(92vw,26rem)] overflow-hidden rounded-2xl border border-white/10 bg-[#0b0b12]/95 shadow-2xl backdrop-blur"
        >
          <div className="flex items-center gap-2 border-b border-white/10 px-3 py-2">
            <LuSearch aria-hidden="true" className="shrink-0 text-white/50" />
            <input
              ref={inputRef}
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              onKeyDown={onInputKeyDown}
              placeholder="搜索文章和建站日志…"
              aria-label="搜索关键词"
              className="w-full bg-transparent text-sm text-white outline-none placeholder:text-white/40"
            />
          </div>

          <div className="max-h-80 overflow-y-auto">
            {status === "loading" && (
              <p className="px-4 py-3 text-sm text-white/60">搜索中…</p>
            )}
            {status === "unconfigured" && (
              <p className="px-4 py-3 text-sm text-white/60">搜索服务还没配置好。</p>
            )}
            {status === "error" && (
              <p className="px-4 py-3 text-sm text-rose-300">{error}</p>
            )}
            {status === "done" && hits.length === 0 && (
              <p className="px-4 py-3 text-sm text-white/60">
                没有找到「{query.trim()}」相关的内容。
              </p>
            )}
            {hits.map((item, index) => (
              <button
                key={`${item.url}-${index}`}
                type="button"
                className={clsx(
                  "block w-full px-4 py-3 text-left transition-colors",
                  index === activeIndex ? "bg-white/10" : "hover:bg-white/5"
                )}
                onMouseEnter={() => setActiveIndex(index)}
                onClick={() => goTo(item.url)}
              >
                <span className="block truncate text-sm text-white">{item.title}</span>
                {item.excerpt && (
                  <span className="mt-0.5 block truncate text-xs text-white/50">
                    {item.excerpt}
                  </span>
                )}
                <span className="mt-0.5 block truncate text-xs text-[#7C5CFF]">
                  {item.url}
                </span>
              </button>
            ))}
          </div>

          <div className="flex items-center gap-1.5 border-t border-white/10 px-4 py-2">
            <span aria-hidden="true" className="h-1.5 w-1.5 rounded-full bg-[#7C5CFF]" />
            <span className="text-xs text-white/40">由 Zilliz 集群提供搜索服务</span>
          </div>
        </form>
      )}
    </div>
  );
};

export default SiteSearch;
