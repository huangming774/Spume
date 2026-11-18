import { notifyAvatarUpdated } from "../lib/siteAvatar.js";

const JSON_HEADERS = { "Content-Type": "application/json", Accept: "application/json" };

async function parseJson(res) {
  const data = await res.json().catch(() => ({}));
  return { ok: res.ok, status: res.status, data };
}

export async function authMe() {
  const res = await fetch("/api/auth/me", {
    credentials: "include",
    headers: { Accept: "application/json" },
  });
  return parseJson(res);
}

export async function authRegister(email, password) {
  const res = await fetch("/api/auth/register", {
    method: "POST",
    credentials: "include",
    headers: JSON_HEADERS,
    body: JSON.stringify({ email, password }),
  });
  return parseJson(res);
}

export async function authLogin(email, password) {
  const res = await fetch("/api/auth/login", {
    method: "POST",
    credentials: "include",
    headers: JSON_HEADERS,
    body: JSON.stringify({ email, password }),
  });
  return parseJson(res);
}

export async function authVerifySecurity(challengeToken, answer) {
  const res = await fetch("/api/auth/verify-security", {
    method: "POST",
    credentials: "include",
    headers: JSON_HEADERS,
    body: JSON.stringify({ challengeToken, answer }),
  });
  return parseJson(res);
}

export async function authLogout() {
  const res = await fetch("/api/auth/logout", {
    method: "POST",
    credentials: "include",
    headers: JSON_HEADERS,
  });
  return parseJson(res);
}

export async function authUpdateProfile(displayName) {
  const res = await fetch("/api/auth/profile", {
    method: "PATCH",
    credentials: "include",
    headers: JSON_HEADERS,
    body: JSON.stringify({ displayName }),
  });
  return parseJson(res);
}

export async function authUploadAvatar(file) {
  const form = new FormData();
  form.append("file", file);
  const res = await fetch("/api/auth/avatar", {
    method: "POST",
    credentials: "include",
    headers: { Accept: "application/json" },
    body: form,
  });
  const result = await parseJson(res);
  if (result.ok && result.data?.user) {
    notifyAvatarUpdated(result.data.user);
  }
  return result;
}
