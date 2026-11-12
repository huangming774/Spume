export async function measureClientNetwork({
  fetchImpl = fetch,
  now = () => performance.now(),
} = {}) {
  const startedAt = now();
  const res = await fetchImpl(`/api/client/network?ts=${Date.now()}`, {
    headers: { Accept: "application/json" },
    cache: "no-store",
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.message || data.error || `HTTP ${res.status}`);
  }
  const latencyMs = Math.max(0, Math.round(now() - startedAt));
  const regionLabel = data.regionLabel || "";
  const ipMasked = data.ipMasked || "";
  const isGenericFallback = regionLabel === "" || regionLabel === "访客";
  const addressLabel = isGenericFallback ? (ipMasked || "访客") : regionLabel;
  return {
    addressLabel,
    regionLabel,
    ipMasked,
    serverTime: data.serverTime || "",
    latencyMs,
  };
}
