import { asList } from "../lib/asList.js";

const DEFAULT_CHANNEL = "guestbook";
const JSON_HEADERS = { Accept: "application/json", "Content-Type": "application/json" };

async function readPayload(res) {
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.message || data.error || `HTTP ${res.status}`);
  }
  return data;
}

function toLegacyRow(item) {
  return {
    id: item.id,
    name: item.name || item.nickname || "匿名",
    content: item.content || "",
    createdAt: item.createdAt,
  };
}

export async function getGuestbook(limit = 50) {
  const pageSize = Math.max(1, Math.min(Number(limit) || 50, 100));
  const res = await fetch(
    `/api/guestbook/messages?page=1&pageSize=${pageSize}&channel=${DEFAULT_CHANNEL}`,
    { credentials: "include", headers: { Accept: "application/json" } },
  );
  const data = await readPayload(res);
  return asList(data).map(toLegacyRow);
}

export async function postGuestbook({ name, content }) {
  const nickname = (name || "").trim().slice(0, 12) || "匿名";
  const message = (content || "").trim().slice(0, 300);
  const res = await fetch("/api/guestbook/messages", {
    method: "POST",
    credentials: "include",
    headers: JSON_HEADERS,
    body: JSON.stringify({
      nickname,
      content: message,
      channel: DEFAULT_CHANNEL,
    }),
  });
  const data = await readPayload(res);
  return toLegacyRow(data.item || data);
}
