import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { createAiImage, fetchImageQuota } from "../services/aiImageApi";

const SIZES = [
  { value: "1024*1024", label: "方图", hint: "1024 × 1024" },
  { value: "1280*720", label: "横图", hint: "1280 × 720" },
  { value: "720*1280", label: "竖图", hint: "720 × 1280" },
];

const friendlyError = (status, data) => {
  switch (data?.error) {
    case "LOGIN_REQUIRED":
      return "请先登录后再生成图片。";
    case "IMAGE_NOT_CONFIGURED":
      return "生图服务还没配置好。";
    case "IMAGE_PROMPT_REJECTED":
      return data.message || "提示词被安全策略拒绝了，换个说法试试。";
    case "RATE_LIMITED":
      return "操作太频繁了，歇一会儿再试。";
    case "DAILY_LIMIT_EXCEEDED":
      return "今日生成次数用完啦，明天再来吧。";
    case "R2_NOT_CONFIGURED":
      return "图床还没配置好。";
    default:
      return data?.message || `生成失败（HTTP ${status}），请稍后再试。`;
  }
};

const AiImagePage = () => {
  const [quota, setQuota] = useState(null);
  const [prompt, setPrompt] = useState("");
  const [size, setSize] = useState("1024*1024");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState(null);

  const loadQuota = useCallback(async () => {
    try {
      const res = await fetchImageQuota();
      if (res.ok) setQuota(res.data);
    } catch {
      // 配额拿不到不挡页面，生成时再报错。
    }
  }, []);

  useEffect(() => {
    loadQuota();
  }, [loadQuota]);

  const remainingText =
    quota == null
      ? "…"
      : quota.unlimited || quota.remaining < 0
        ? "不限量"
        : `今日剩 ${quota.remaining} 张`;

  const onSubmit = async (event) => {
    event.preventDefault();
    if (busy) return;
    if (!prompt.trim()) {
      setError("先写一句提示词吧。");
      return;
    }
    setBusy(true);
    setError("");
    setResult(null);
    try {
      const res = await createAiImage(prompt.trim(), size);
      if (!res.ok) {
        setError(friendlyError(res.status, res.data));
      } else {
        setResult(res.data.image || null);
        setPrompt("");
      }
      if (res.data && typeof res.data === "object" && "limit" in res.data) {
        setQuota(res.data);
      }
    } catch (e) {
      setError(e.message || "生成失败，请稍后再试。");
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="relative min-h-screen overflow-hidden bg-[#fef6f1] pt-24 text-[#241322]">
      <div
        className="pointer-events-none fixed inset-0 -z-10 bg-[radial-gradient(circle_at_18%_18%,rgba(255,210,225,0.5),transparent_38%),radial-gradient(circle_at_84%_12%,rgba(196,210,255,0.45),transparent_36%),radial-gradient(circle_at_50%_88%,rgba(255,238,196,0.4),transparent_40%),linear-gradient(135deg,#fff7f1_0%,#fbeef3_48%,#eef5fb_100%)]"
        aria-hidden="true"
      />

      <header className="container mx-auto px-5 pb-10 md:px-10">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="font-general text-xs uppercase tracking-[0.45em] text-[#b76e79]">
              AI Image Workshop
            </p>
            <h1 className="mt-4 font-zentry text-5xl font-black uppercase leading-none text-[#241322] md:text-7xl">
              生成工坊
            </h1>
          </div>
          <div className="flex items-center gap-3">
            <span className="rounded-full border border-white/70 bg-white/60 px-4 py-2 text-xs font-bold text-[#5f4b52] backdrop-blur">
              {quota?.model || "AI 生图"} · {remainingText}
            </span>
            <Link
              to="/ai-gallery"
              className="rounded-full border border-[#241322]/15 px-4 py-2 text-xs font-bold uppercase tracking-[0.18em] text-[#241322]"
            >
              去广场看看
            </Link>
          </div>
        </div>
      </header>

      <section className="container mx-auto grid gap-6 pb-24 md:px-10 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
        {/* 生成表单 */}
        <form onSubmit={onSubmit} className="rounded-3xl border border-white/60 bg-white/55 p-8 backdrop-blur-md">
          {quota && !quota.isLogin && (
            <p className="mb-5 rounded-2xl border border-[#ff8fab]/30 bg-[#ff8fab]/10 px-4 py-3 text-sm text-[#8a4050]">
              生成图片需要先
              <Link to="/login" className="mx-1 font-bold underline">
                登录
              </Link>
              （防止额度被刷爆）。
            </p>
          )}
          {quota && quota.isLogin && quota.imageEnabled === false && (
            <p className="mb-5 rounded-2xl border border-[#ff8fab]/30 bg-[#ff8fab]/10 px-4 py-3 text-sm text-[#8a4050]">
              生图服务还没配置好，站长配好模型就能用。
            </p>
          )}

          <label className="font-general text-xs uppercase tracking-[0.32em] text-[#b76e79]" htmlFor="ai-image-prompt">
            Prompt 提示词
          </label>
          <textarea
            id="ai-image-prompt"
            value={prompt}
            onChange={(event) => setPrompt(event.target.value)}
            placeholder="一只折纸小船漂在湖面，水彩风…"
            rows={5}
            className="mt-3 w-full resize-y rounded-2xl border border-white/70 bg-white/70 px-4 py-3 text-base text-[#241322] outline-none placeholder:text-[#b9a6ad] focus:border-[#ff8fab]/60"
          />

          <p className="mt-6 font-general text-xs uppercase tracking-[0.32em] text-[#b76e79]">尺寸</p>
          <div className="mt-3 flex flex-wrap gap-3">
            {SIZES.map((item) => (
              <button
                key={item.value}
                type="button"
                onClick={() => setSize(item.value)}
                className={`rounded-full px-5 py-2.5 text-sm font-bold transition ${
                  size === item.value
                    ? "bg-[#241322] text-[#ffe7ef]"
                    : "border border-[#241322]/15 bg-white/70 text-[#5f4b52]"
                }`}
              >
                {item.label}
                <span className="ml-2 text-xs font-medium opacity-70">{item.hint}</span>
              </button>
            ))}
          </div>

          {error && (
            <p className="mt-5 rounded-2xl border border-[#ff8fab]/30 bg-[#ff8fab]/10 px-4 py-3 text-sm text-[#8a4050]">
              {error}
            </p>
          )}

          <button
            type="submit"
            disabled={busy}
            className="mt-6 w-full rounded-full bg-[#241322] px-6 py-3.5 text-sm font-bold uppercase tracking-[0.22em] text-[#ffe7ef] disabled:cursor-not-allowed disabled:opacity-50"
          >
            {busy ? "生成中，大约需要一会儿…" : "生成图片"}
          </button>
          <p className="mt-3 text-center text-xs text-[#8d747c]">
            生成的图片会公开进入生成图广场，每人每天有额度。
          </p>
        </form>

        {/* 结果展示 */}
        <div className="rounded-3xl border border-white/60 bg-white/55 p-8 backdrop-blur-md">
          {busy && (
            <div className="flex min-h-[320px] flex-col items-center justify-center gap-3 text-[#5f4b52]">
              <span className="h-10 w-10 animate-spin rounded-full border-4 border-[#ff8fab]/30 border-t-[#ff8fab]" />
              <p className="text-sm">正在作画，请稍候…</p>
            </div>
          )}
          {!busy && result && (
            <figure>
              <img
                src={result.url}
                alt={result.prompt || "AI 生成图片"}
                className="w-full rounded-2xl border border-white/80 object-contain"
              />
              <figcaption className="mt-4 space-y-1 text-sm text-[#5f4b52]">
                <p className="font-bold text-[#241322]">{result.size || size}</p>
                <p className="text-xs text-[#8d747c]">{result.mimeType}</p>
              </figcaption>
            </figure>
          )}
          {!busy && !result && (
            <div className="flex min-h-[320px] flex-col items-center justify-center gap-3 text-center text-[#5f4b52]">
              <p className="font-general text-xs uppercase tracking-[0.36em] text-[#b76e79]">Preview</p>
              <p className="max-w-xs text-sm leading-relaxed">
                左边写下你想要的画面，选好尺寸点生成。画好后在这里展示，也会同步到生成图广场。
              </p>
            </div>
          )}
        </div>
      </section>
    </div>
  );
};

export default AiImagePage;
