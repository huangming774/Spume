const JSON_HEADERS = { "Content-Type": "application/json", Accept: "application/json" };

async function parseJson(res) {
  const data = await res.json().catch(() => ({}));
  return { ok: res.ok, status: res.status, data };
}

/** GET /api/ai/image → 配额快照 {limit, used, remaining, isLogin, imageEnabled, canGenerate, model, ...} */
export async function fetchImageQuota() {
  const res = await fetch("/api/ai/image", {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return parseJson(res);
}

/** POST /api/ai/image {prompt, size} → 成功带 image: {url, mimeType, size, width, height} 和配额字段。 */
export async function createAiImage(prompt, size) {
  const res = await fetch("/api/ai/image", {
    method: "POST",
    credentials: "include",
    headers: JSON_HEADERS,
    body: JSON.stringify({ prompt, size }),
  });
  return parseJson(res);
}
