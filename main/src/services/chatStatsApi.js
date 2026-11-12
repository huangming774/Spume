// Mock 数据，仅当 URL 含 ?mock=1 时启用（本地预览）
const MOCK_STATS = {
  model: "deepseek-v4-flash",
  configured: true,
  summary: {
    todayTotal: 47,
    todaySuccess: 42,
    periodTotal: 583,
    periodSuccess: 521,
    successRateText: "89.4%",
  },
  daily: [
    { date: "2026-06-09", total: 38, success: 35, upstreamError: 1, quotaDenied: 0, rateDenied: 2, guestCalls: 22, userCalls: 16 },
    { date: "2026-06-10", total: 42, success: 38, upstreamError: 2, quotaDenied: 1, rateDenied: 1, guestCalls: 25, userCalls: 17 },
    { date: "2026-06-11", total: 51, success: 46, upstreamError: 2, quotaDenied: 1, rateDenied: 2, guestCalls: 30, userCalls: 21 },
    { date: "2026-06-12", total: 33, success: 30, upstreamError: 1, quotaDenied: 0, rateDenied: 2, guestCalls: 18, userCalls: 15 },
    { date: "2026-06-13", total: 60, success: 54, upstreamError: 3, quotaDenied: 1, rateDenied: 2, guestCalls: 35, userCalls: 25 },
    { date: "2026-06-14", total: 48, success: 44, upstreamError: 2, quotaDenied: 1, rateDenied: 1, guestCalls: 27, userCalls: 21 },
    { date: "2026-06-15", total: 55, success: 49, upstreamError: 3, quotaDenied: 2, rateDenied: 1, guestCalls: 32, userCalls: 23 },
    { date: "2026-06-16", total: 41, success: 37, upstreamError: 2, quotaDenied: 0, rateDenied: 2, guestCalls: 24, userCalls: 17 },
    { date: "2026-06-17", total: 39, success: 35, upstreamError: 2, quotaDenied: 1, rateDenied: 1, guestCalls: 21, userCalls: 18 },
    { date: "2026-06-18", total: 44, success: 40, upstreamError: 2, quotaDenied: 1, rateDenied: 1, guestCalls: 26, userCalls: 18 },
    { date: "2026-06-19", total: 58, success: 52, upstreamError: 3, quotaDenied: 1, rateDenied: 2, guestCalls: 33, userCalls: 25 },
    { date: "2026-06-20", total: 50, success: 45, upstreamError: 2, quotaDenied: 1, rateDenied: 2, guestCalls: 28, userCalls: 22 },
    { date: "2026-06-21", total: 37, success: 34, upstreamError: 1, quotaDenied: 0, rateDenied: 2, guestCalls: 20, userCalls: 17 },
    { date: "2026-06-22", total: 47, success: 42, upstreamError: 2, quotaDenied: 1, rateDenied: 2, guestCalls: 26, userCalls: 21 },
  ],
  hourlyToday: Array.from({ length: 24 }, (_, h) => {
    const hh = String(h).padStart(2, "0");
    const wave = Math.sin(((h - 6) / 24) * Math.PI * 2) * 6 + 8;
    const total = h < 6 ? Math.max(0, Math.round(wave * 0.3)) : Math.max(0, Math.round(wave));
    const success = Math.max(0, total - (h % 3 === 0 ? 1 : 0));
    return { hour: hh, total, success };
  }),
};

function isMockMode() {
  if (typeof window === "undefined") return false;
  const sp = new URLSearchParams(window.location.search);
  return sp.get("mock") === "1";
}

export async function fetchChatStats(days = 14) {
  if (isMockMode()) {
    return Promise.resolve({
      ...MOCK_STATS,
      summary: {
        ...MOCK_STATS.summary,
        todaySuccess: 42 + Math.floor(Math.random() * 3),
      },
    });
  }
  const res = await fetch(`/api/chat/stats?days=${days}`, {
    headers: { Accept: "application/json" },
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.message || data.error || "加载统计失败");
  }
  return data;
}
