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
