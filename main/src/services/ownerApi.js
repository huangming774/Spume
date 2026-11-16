const JSON_HEADERS = { Accept: "application/json" };

async function parseResponse(res) {
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.message || data.error || `HTTP ${res.status}`);
  }
  return data;
}

export async function fetchOwnerStatus() {
  const res = await fetch("/api/owner/status", {
    credentials: "include",
    headers: JSON_HEADERS,
  });
  return parseResponse(res);
}

export async function fetchOwnerEmails() {
  const res = await fetch("/api/owner/emails", {
    credentials: "include",
    headers: JSON_HEADERS,
  });
  return parseResponse(res);
}

export async function fetchOwnerDrafts() {
  const res = await fetch("/api/owner/drafts", {
    credentials: "include",
    headers: JSON_HEADERS,
  });
  return parseResponse(res);
}

export async function createOwnerDraft(payload) {
  const res = await fetch("/api/owner/drafts", {
    method: "POST",
    credentials: "include",
    headers: {
      ...JSON_HEADERS,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return parseResponse(res);
}

export async function publishOwnerArticle(payload) {
  const res = await fetch("/api/owner/publish", {
    method: "POST",
    credentials: "include",
    headers: {
      ...JSON_HEADERS,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return parseResponse(res);
}

export async function publishOwnerGalleryImage(payload) {
  const res = await fetch("/api/owner/gallery/images", {
    method: "POST",
    credentials: "include",
    headers: {
      ...JSON_HEADERS,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return parseResponse(res);
}

export async function markOwnerNotificationRead(id) {
  const res = await fetch(`/api/owner/notifications/${encodeURIComponent(id)}/read`, {
    method: "PATCH",
    credentials: "include",
    headers: JSON_HEADERS,
  });
  return parseResponse(res);
}

export async function publishOwnerFriend(payload) {
  const res = await fetch("/api/owner/friends", {
    method: "POST",
    credentials: "include",
    headers: {
      ...JSON_HEADERS,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return parseResponse(res);
}

export async function publishOwnerMoment(payload) {
  const res = await fetch("/api/owner/moments", {
    method: "POST",
    credentials: "include",
    headers: {
      ...JSON_HEADERS,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return parseResponse(res);
}

export async function createOwnerFixedAnswer(payload) {
  const res = await fetch("/api/owner/ai/fixed-answers", {
    method: "POST",
    credentials: "include",
    headers: {
      ...JSON_HEADERS,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(payload),
  });
  return parseResponse(res);
}

export function isPublicImageURL(value) {
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}

export async function uploadOwnerAsset(file, { kind, album = "" }) {
  const form = new FormData();
  form.append("file", file);
  form.append("kind", kind);
  if (album) {
    form.append("album", album);
  }

  const res = await fetch("/api/owner/assets", {
    method: "POST",
    credentials: "include",
    body: form,
  });
  return parseResponse(res);
}

export async function uploadOwnerImage(file) {
  const form = new FormData();
  form.append("file", file);

  const res = await fetch("/api/owner/uploads", {
    method: "POST",
    credentials: "include",
    body: form,
  });
  return parseResponse(res);
}

function ownerJson(url, method, payload) {
  return fetch(url, {
    method,
    credentials: "include",
    headers: {
      ...JSON_HEADERS,
      ...(payload ? { "Content-Type": "application/json" } : {}),
    },
    body: payload ? JSON.stringify(payload) : undefined,
  }).then(parseResponse);
}

export function deleteOwnerEntry(id, kind) {
  return ownerJson(
    `/api/owner/content/${encodeURIComponent(id)}?kind=${encodeURIComponent(kind)}`,
    "DELETE",
  );
}

export function publishOwnerJournal(payload) {
  return ownerJson("/api/owner/journal", "POST", payload);
}

export function deleteOwnerJournal(slug) {
  return ownerJson(`/api/owner/journal?slug=${encodeURIComponent(slug)}`, "DELETE");
}

export function deleteOwnerGalleryImage(payload) {
  return ownerJson("/api/owner/gallery/images", "DELETE", payload);
}

export function deleteOwnerMoment(payload) {
  return ownerJson("/api/owner/moments", "DELETE", payload);
}

export function publishOwnerAbout(payload) {
  return ownerJson("/api/owner/about", "POST", payload);
}

export function deleteOwnerAbout(id) {
  return ownerJson("/api/owner/about", "DELETE", { id });
}

export function publishOwnerBili(payload) {
  return ownerJson("/api/owner/bili", "POST", payload);
}

export function deleteOwnerBili(bvid) {
  return ownerJson("/api/owner/bili", "DELETE", { bvid });
}

export function publishOwnerProject(payload) {
  return ownerJson("/api/owner/projects", "POST", payload);
}

export function deleteOwnerProject(id) {
  return ownerJson("/api/owner/projects", "DELETE", { id });
}
