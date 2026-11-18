import { cosAsset } from "./cosAsset.js";

export const defaultSiteAvatarUrl = cosAsset("1.png");

export function siteAvatarFromProfile(data) {
  const avatar = typeof data?.avatar === "string" ? data.avatar.trim() : "";
  try {
    const url = new URL(avatar);
    if (url.protocol === "http:" || url.protocol === "https:") return avatar;
  } catch {
    return defaultSiteAvatarUrl;
  }
  return defaultSiteAvatarUrl;
}

export function siteDisplayNameFromProfile(data) {
  const name = typeof data?.displayName === "string" ? data.displayName.trim() : "";
  return name;
}

/** 头像更换的全站广播：同页听事件，跨标签页听 storage。 */
export const AVATAR_UPDATED_EVENT = "blog-avatar-updated";
export const AVATAR_REV_KEY = "spume-avatar-rev";

export function notifyAvatarUpdated(user) {
  try {
    window.dispatchEvent(new CustomEvent(AVATAR_UPDATED_EVENT, { detail: { user } }));
    window.dispatchEvent(new CustomEvent("blog-auth-profile-updated", { detail: { user } }));
  } catch {
    /* 非浏览器环境忽略 */
  }
  try {
    localStorage.setItem(AVATAR_REV_KEY, String(Date.now()));
  } catch {
    /* 存储不可用时只靠同页事件 */
  }
}

/** 订阅头像更换（同页事件 + 跨标签页 storage），返回取消函数。 */
export function onAvatarUpdated(callback) {
  const handler = () => callback();
  const storageHandler = (event) => {
    if (event.key === AVATAR_REV_KEY) callback();
  };
  window.addEventListener(AVATAR_UPDATED_EVENT, handler);
  window.addEventListener("storage", storageHandler);
  return () => {
    window.removeEventListener(AVATAR_UPDATED_EVENT, handler);
    window.removeEventListener("storage", storageHandler);
  };
}

let siteProfileCache = null;

/** 取站长公开头像与昵称，带缓存；头像更换后自动失效重取。 */
export function fetchSiteProfile({ force = false } = {}) {
  if (siteProfileCache && !force) return siteProfileCache;
  siteProfileCache = fetch("/api/site/avatar", {
    headers: { Accept: "application/json" },
    cache: "no-store",
  })
    .then((res) => (res.ok ? res.json() : null))
    .then((data) => ({
      avatar: siteAvatarFromProfile(data),
      displayName: siteDisplayNameFromProfile(data),
    }))
    .catch(() => ({ avatar: defaultSiteAvatarUrl, displayName: "" }));
  return siteProfileCache;
}

export function invalidateSiteProfile() {
  siteProfileCache = null;
}
