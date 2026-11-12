import { useEffect, useState } from "react";
import { getGuestbook, postGuestbook } from "../services/guestbookApi";

const formatTime = (iso) => {
  try {
    return new Date(iso).toLocaleString("en-GB", {
      dateStyle: "medium",
      timeStyle: "short",
    });
  } catch {
    return iso;
  }
};

const Guestbook = ({ theme = "dark" }) => {
  const candy = theme === "candy";
  const [entries, setEntries] = useState([]);
  const [name, setName] = useState("");
  const [content, setContent] = useState("");
  const [loading, setLoading] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [notice, setNotice] = useState("");

  const load = async () => {
    setLoading(true);
    setNotice("");
    try {
      const items = await getGuestbook(50);
      setEntries(items);
    } catch (e) {
      setEntries([]);
      setNotice(e.message || "留言加载失败，请稍后再试。");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const onSubmit = async (e) => {
    e.preventDefault();
    if (!content.trim()) return;
    setSubmitting(true);
    setNotice("");
    try {
      const row = await postGuestbook({
        name: name.trim() || "匿名",
        content,
      });
      setEntries((prev) => [row, ...prev].slice(0, 50));
      setContent("");
      setNotice("留言已发布。");
    } catch (e) {
      setNotice(e.message || "留言发布失败，请稍后再试。");
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <section
      id="guestbook"
      className={candy ? "pb-24 pt-8" : "bg-violet-300/20 pb-24 pt-12"}
    >
      <div className="container mx-auto px-3 md:px-10">
        <header className="mb-8 px-2">
          <p
            className={
              candy
                ? "font-mono text-[10px] uppercase tracking-[0.35em] text-[#00C2FF]/90"
                : "font-circular-web text-xs uppercase tracking-[0.35em] text-blue-200/80"
            }
          >
            留言板
          </p>
          <h2
            className={
              candy
                ? "mt-2 font-zentry text-3xl font-black uppercase text-[#2D2A3A] md:text-5xl"
                : "mt-2 font-zentry text-3xl font-black uppercase text-blue-50 md:text-5xl"
            }
          >
            留下一张小纸条
          </h2>
          <p
            className={
              candy
                ? "mt-3 max-w-xl text-sm leading-relaxed text-[#2D2A3A]/70"
                : "mt-3 max-w-xl text-sm leading-relaxed text-blue-50/80"
            }
          >
            写下你的来访痕迹，我会在这里认真收下。
          </p>
        </header>

        <form
          onSubmit={onSubmit}
          className={
            candy
              ? "mx-2 mb-10 max-w-2xl rounded-2xl border border-[#7C5CFF]/20 bg-white/55 p-5 shadow-[0_8px_32px_rgba(124,92,255,0.08)] backdrop-blur-xl"
              : "mx-2 mb-10 max-w-2xl rounded-xl border border-white/15 bg-black/40 p-5 backdrop-blur-sm"
          }
        >
          <label
            className={
              candy
                ? "block text-xs font-bold uppercase tracking-wider text-[#7C5CFF]/80"
                : "block text-xs font-bold uppercase tracking-wider text-blue-50/70"
            }
          >
            昵称
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            maxLength={32}
            placeholder="匿名"
            className={
              candy
                ? "mt-2 w-full rounded-lg border border-[#7C5CFF]/15 bg-white/70 px-3 py-2 text-sm text-[#2D2A3A] outline-none focus:border-[#7C5CFF]"
                : "mt-2 w-full rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-blue-50 outline-none focus:border-yellow-300"
            }
          />
          <label
            className={
              candy
                ? "mt-4 block text-xs font-bold uppercase tracking-wider text-[#7C5CFF]/80"
                : "mt-4 block text-xs font-bold uppercase tracking-wider text-blue-50/70"
            }
          >
            留言
          </label>
          <textarea
            value={content}
            onChange={(e) => setContent(e.target.value)}
            maxLength={500}
            rows={4}
            required
            placeholder="写点想说的话吧…"
            className={
              candy
                ? "mt-2 w-full resize-y rounded-lg border border-[#7C5CFF]/15 bg-white/70 px-3 py-2 text-sm text-[#2D2A3A] outline-none focus:border-[#7C5CFF]"
                : "mt-2 w-full resize-y rounded-lg border border-white/10 bg-white/5 px-3 py-2 text-sm text-blue-50 outline-none focus:border-yellow-300"
            }
          />
          <button
            type="submit"
            disabled={submitting}
            className={
              candy
                ? "mt-4 rounded-full border border-[#7C5CFF]/50 bg-[#7C5CFF] px-6 py-2 text-xs font-bold uppercase text-white disabled:opacity-60"
                : "mt-4 rounded-full bg-yellow-400 px-6 py-2 text-xs font-bold uppercase text-black disabled:opacity-60"
            }
          >
            {submitting ? "发布中…" : "发布留言"}
          </button>
          {notice && (
            <p className={`mt-3 text-xs ${candy ? "text-[#7C5CFF]" : "text-yellow-200"}`}>
              {notice}
            </p>
          )}
        </form>

        <div className="grid grid-cols-1 gap-4 p-2 sm:grid-cols-2 lg:grid-cols-3">
          {loading ? (
            <p className={`text-sm ${candy ? "text-[#2D2A3A]/50" : "text-white/50"}`}>
              正在加载留言…
            </p>
          ) : (
            entries.map((row) => (
              <article
                key={row.id}
                className={
                  candy
                    ? "h-auto rounded-2xl border border-[#FF6BAA]/15 bg-white/55 p-5 text-[#2D2A3A] shadow-sm backdrop-blur-md"
                    : "h-auto rounded-xl border border-white/10 bg-black/50 p-5 text-blue-50"
                }
              >
                <div className="flex items-baseline justify-between gap-2">
                  <span
                    className={
                      candy ? "text-sm font-bold text-[#FF6BAA]" : "text-sm font-bold text-yellow-300"
                    }
                  >
                    {row.name}
                  </span>
                  <time
                    className={`shrink-0 text-[10px] ${candy ? "text-[#2D2A3A]/45" : "text-white/50"}`}
                  >
                    {formatTime(row.createdAt)}
                  </time>
                </div>
                <p
                  className={
                    candy
                      ? "mt-3 break-words text-sm leading-relaxed text-[#2D2A3A]/80"
                      : "mt-3 break-words text-sm leading-relaxed text-white/85"
                  }
                >
                  {row.content}
                </p>
              </article>
            ))
          )}
        </div>
      </div>
    </section>
  );
};

export default Guestbook;
