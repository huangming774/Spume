import { apiUrl } from "../lib/apiBase.js";

/**
 * 「创造纪事」提交时间轴数据源：后端带 token 拉 GitHub 并缓存 10 分钟。
 * 返回 { configured, available, reason?, repo?, branch?, commits: [...] }；
 * 网络失败时返回 available=false，由页面展示「暂时无法同步」。
 */
export async function getRepoCommits() {
  try {
    const res = await fetch(apiUrl("/api/client/repo-commits"), {
      headers: { Accept: "application/json" },
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok) {
      return {
        configured: true,
        available: false,
        reason: data.message || data.error || `HTTP ${res.status}`,
        commits: [],
      };
    }
    return {
      configured: Boolean(data.configured),
      available: Boolean(data.available),
      reason: data.reason || "",
      repo: data.repo || "",
      branch: data.branch || "",
      commits: Array.isArray(data.commits) ? data.commits : [],
    };
  } catch (error) {
    return {
      configured: true,
      available: false,
      reason: String(error?.message || error),
      commits: [],
    };
  }
}
