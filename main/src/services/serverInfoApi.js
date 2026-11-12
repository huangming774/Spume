export async function fetchServerInfo() {
  const res = await fetch("/api/server/info", {
    headers: { Accept: "application/json" },
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.message || data.error || "加载服务器信息失败");
  }
  return data;
}
