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
