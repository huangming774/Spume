const JSON_HEADERS = { "Content-Type": "application/json", Accept: "application/json" };
const DEFAULT_CHANNEL = "guestbook";

function withChannel(body, channel = DEFAULT_CHANNEL) {
  return { ...body, channel };
}

async function parseJson(res) {
  const data = await res.json().catch(() => ({}));
  return { ok: res.ok, status: res.status, data };
}

export async function fetchGuestbookMessages(
  page = 1,
  pageSize = 20,
  { channel = DEFAULT_CHANNEL } = {}
) {
  const res = await fetch(
    `/api/guestbook/messages?page=${page}&pageSize=${pageSize}&channel=${encodeURIComponent(channel)}`,
    { credentials: "include", headers: { Accept: "application/json" } }
  );
  return parseJson(res);
}

export async function postGuestbookMessage(body, { channel = DEFAULT_CHANNEL } = {}) {
  const res = await fetch("/api/guestbook/messages", {
    method: "POST",
    credentials: "include",
    headers: JSON_HEADERS,
    body: JSON.stringify(withChannel(body, channel)),
  });
  return parseJson(res);
}

export function postGuestbookReply(
  parentId,
  content,
  { channel = DEFAULT_CHANNEL } = {}
) {
  return postGuestbookMessage({ parentId, content }, { channel });
}

export async function hideGuestbookMessage(id) {
  const res = await fetch(`/api/guestbook/messages/${id}`, {
    method: "PATCH",
    credentials: "include",
    headers: JSON_HEADERS,
    body: JSON.stringify({ status: "hidden" }),
  });
  return parseJson(res);
}

export async function deleteGuestbookMessage(id) {
  const res = await fetch(`/api/guestbook/messages/${id}`, {
    method: "DELETE",
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return parseJson(res);
}
