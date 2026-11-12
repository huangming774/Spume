import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";

const PAGE_SIZE = 60;

function formatDate(iso) {
  if (!iso) return "";
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return "";
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  const h = String(d.getHours()).padStart(2, "0");
  const min = String(d.getMinutes()).padStart(2, "0");
  return `${y}.${m}.${day} ${h}:${min}`;
}

// Decide the mosaic span of each tile from its position in the feed. Mixing
// 1×1, 1×2, 2×1 and 2×2 tiles produces the irregular collage layout from the
// reference image. We base it on `index` rather than something derived from the
// data so the layout stays stable when new pages arrive.
function tileSpan(index) {
  const cycle = index % 11;
  if (cycle === 0) return "col-span-2 row-span-2"; // big square
  if (cycle === 3) return "col-span-2 row-span-1"; // wide
  if (cycle === 7) return "col-span-1 row-span-2"; // tall
  if (cycle === 9) return "col-span-2 row-span-2"; // big square again
  return "col-span-1 row-span-1";
}

const AiGalleryPage = () => {
  const [items, setItems] = useState([]);
  const [cursor, setCursor] = useState(null);
  const [loading, setLoading] = useState(true);
  const [reachedEnd, setReachedEnd] = useState(false);
  const [error, setError] = useState(null);
  const [active, setActive] = useState(null); // selected item for modal
  const sentinelRef = useRef(null);

  const fetchPage = useCallback(async (before) => {
    const params = new URLSearchParams({ limit: String(PAGE_SIZE) });
    if (before) params.set("before", before);
    const res = await fetch(`/api/ai/image/gallery?${params.toString()}`, {
      headers: { Accept: "application/json" },
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  }, []);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    fetchPage(null)
      .then((data) => {
        if (cancelled) return;
        setItems(data.items || []);
        setCursor(data.nextCursor || null);
        setReachedEnd(!data.nextCursor);
      })
      .catch((err) => {
        if (cancelled) return;
        setError(err.message || "load failed");
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [fetchPage]);

  // Infinite scroll
  useEffect(() => {
    if (reachedEnd || loading || !cursor) return undefined;
    const node = sentinelRef.current;
    if (!node) return undefined;
    if (typeof window === "undefined" || !("IntersectionObserver" in window)) return undefined;

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return;
          setLoading(true);
          fetchPage(cursor)
            .then((data) => {
              setItems((prev) => [...prev, ...(data.items || [])]);
              setCursor(data.nextCursor || null);
              setReachedEnd(!data.nextCursor);
            })
            .catch((err) => setError(err.message || "load failed"))
            .finally(() => setLoading(false));
        });
      },
      { rootMargin: "300px 0px" }
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [cursor, reachedEnd, loading, fetchPage]);

  // Lock body scroll + ESC to close while modal is open
  useEffect(() => {
    if (!active) return undefined;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    const onKey = (e) => {
      if (e.key === "Escape") setActive(null);
    };
    window.addEventListener("keydown", onKey);
    return () => {
      document.body.style.overflow = prev;
      window.removeEventListener("keydown", onKey);
    };
  }, [active]);

  const tiles = useMemo(
    () => items.map((it, i) => ({ ...it, _span: tileSpan(i) })),
    [items]
  );

  return (
    <div className="relative min-h-screen overflow-hidden bg-[#fef6f1] pt-24 text-[#241322]">
      {/* Soft pastel ambience */}
      <div
        className="pointer-events-none fixed inset-0 -z-10 bg-[radial-gradient(circle_at_18%_18%,rgba(255,210,225,0.5),transparent_38%),radial-gradient(circle_at_84%_12%,rgba(196,210,255,0.45),transparent_36%),radial-gradient(circle_at_50%_88%,rgba(255,238,196,0.4),transparent_40%),linear-gradient(135deg,#fff7f1_0%,#fbeef3_48%,#eef5fb_100%)]"
        aria-hidden="true"
      />

      <header className="container mx-auto px-5 pb-10 md:px-10">
        <p className="font-general text-xs uppercase tracking-[0.45em] text-[#b76e79]">
          AI Image Plaza
        </p>
        <h1 className="mt-4 font-zentry text-5xl font-black uppercase leading-none text-[#241322] md:text-7xl">
          生成图
          <span className="block text-[#ff8fab]">Plaza</span>
        </h1>
      </header>

      <section className="container mx-auto px-5 pb-24 md:px-10">
        {error && !items.length && (
          <div className="rounded-3xl border border-white/60 bg-white/55 p-10 text-center text-[#5f4b52] backdrop-blur-md">
            <p className="font-general text-xs uppercase tracking-[0.36em] text-[#b76e79]">Load failed</p>
            <p className="mt-3 text-base">广场数据暂时拉不到，请稍后刷新再试。</p>
          </div>
        )}

        {!error && !loading && items.length === 0 && (
          <div className="rounded-3xl border border-white/60 bg-white/55 p-12 text-center text-[#5f4b52] backdrop-blur-md">
            <p className="font-general text-xs uppercase tracking-[0.36em] text-[#b76e79]">Empty</p>
            <p className="mt-3 text-base">还没有人在工坊生成图片。</p>
            <Link
              to="/app"
              className="mt-5 inline-flex items-center gap-2 rounded-full bg-[#241322] px-5 py-2.5 text-xs font-bold uppercase tracking-[0.2em] text-[#ffe7ef]"
            >
              去工坊抢首发
            </Link>
          </div>
        )}

        {/* Mosaic grid with mixed tile spans */}
        {tiles.length > 0 && (
          <div className="grid auto-rows-[8rem] grid-cols-2 gap-3 [grid-auto-flow:dense] md:auto-rows-[10rem] md:grid-cols-4 md:gap-4 lg:grid-cols-6">
            {tiles.map((it, idx) => (
              <button
                key={`${it.imageUrl}-${idx}`}
                type="button"
                onClick={() => setActive(it)}
                aria-label="查看提示词与作者"
                className={`iridescent-tile group relative ${it._span} overflow-hidden rounded-2xl focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-4 focus-visible:outline-[#ff8fab]`}
              >
                <img
                  src={it.imageUrl}
                  alt={it.prompt || "AI generated image"}
                  loading={idx < 6 ? "eager" : "lazy"}
                  className="block size-full object-cover transition duration-700 group-hover:scale-[1.04]"
                />
                {/* subtle scrim on hover so users notice it's clickable */}
                <span
                  aria-hidden="true"
                  className="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/35 via-transparent to-transparent opacity-0 transition duration-500 group-hover:opacity-100"
                />

                {/* always-on hint: tiny breathing dot bottom-left */}
                <span
                  aria-hidden="true"
                  className="ai-tile-dot pointer-events-none absolute bottom-2.5 left-2.5 z-10 h-1.5 w-1.5 rounded-full bg-white/85 shadow-[0_0_8px_rgba(255,255,255,0.85)]"
                />

                {/* hover hint: corner chip with prompt icon — opens the modal, not the full prompt */}
                <span
                  aria-hidden="true"
                  className="pointer-events-none absolute right-2.5 top-2.5 z-10 flex items-center gap-1 rounded-full bg-white/85 px-2 py-1 font-general text-[9px] uppercase tracking-[0.18em] text-[#241322] opacity-0 shadow-[0_4px_14px_rgba(0,0,0,0.18)] backdrop-blur-md transition-all duration-300 group-hover:translate-y-0 group-hover:opacity-100 translate-y-[-4px]"
                >
                  <svg viewBox="0 0 12 12" className="h-2.5 w-2.5" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round">
                    <circle cx="5" cy="5" r="3.2" />
                    <path d="M7.6 7.6L10 10" />
                  </svg>
                  Prompt
                </span>
              </button>
            ))}
          </div>
        )}

        <div ref={sentinelRef} aria-hidden="true" className="h-12" />

        {loading && items.length > 0 && (
          <p className="py-6 text-center font-general text-xs uppercase tracking-[0.36em] text-[#b76e79]/70">
            Loading more …
          </p>
        )}

        {reachedEnd && items.length > 0 && (
          <p className="py-8 text-center font-general text-xs uppercase tracking-[0.36em] text-[#b76e79]/60">
            — End of plaza —
          </p>
        )}
      </section>

      {/* Modal: image + prompt + author */}
      {active && (
        <div
          role="dialog"
          aria-modal="true"
          className="fixed inset-0 z-[100] flex items-center justify-center bg-[#1a0c14]/85 p-4 backdrop-blur-sm md:p-10"
          onClick={() => setActive(null)}
        >
          <button
            type="button"
            aria-label="Close"
            onClick={() => setActive(null)}
            className="absolute right-6 top-6 rounded-full border border-white/30 bg-white/15 px-4 py-2 font-general text-xs uppercase tracking-[0.28em] text-white backdrop-blur-md transition hover:bg-white/25"
          >
            Close ✕
          </button>
          <div
            className="relative flex max-h-[88vh] w-full max-w-5xl flex-col gap-5 overflow-hidden rounded-3xl bg-white/95 p-4 shadow-[0_40px_120px_rgba(0,0,0,0.45)] md:flex-row md:p-6"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex-1 overflow-hidden rounded-2xl bg-[#f3e9e3]">
              <img
                src={active.imageUrl}
                alt={active.prompt || "AI generated image"}
                className="block max-h-[80vh] w-full object-contain"
              />
            </div>
            <div className="flex w-full flex-col gap-4 md:w-72">
              <div>
                <p className="font-general text-[10px] uppercase tracking-[0.36em] text-[#b76e79]">
                  Prompt
                </p>
                <p className="mt-2 max-h-64 overflow-y-auto whitespace-pre-wrap text-sm leading-relaxed text-[#241322]">
                  {active.prompt}
                </p>
              </div>
              <div>
                <p className="font-general text-[10px] uppercase tracking-[0.36em] text-[#b76e79]">
                  Author
                </p>
                <p className="mt-2 text-sm font-semibold text-[#241322]">
                  {active.author || "访客"}
                </p>
              </div>
              <div>
                <p className="font-general text-[10px] uppercase tracking-[0.36em] text-[#b76e79]">
                  Created
                </p>
                <p className="mt-2 text-sm text-[#5f4b52]">
                  {formatDate(active.createdAt)}
                </p>
              </div>
              {active.size && (
                <div>
                  <p className="font-general text-[10px] uppercase tracking-[0.36em] text-[#b76e79]">
                    Size
                  </p>
                  <p className="mt-2 text-sm text-[#5f4b52]">{active.size}</p>
                </div>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default AiGalleryPage;
