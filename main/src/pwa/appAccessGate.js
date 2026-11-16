export const PWA_AUTH_HOSTNAME = "app.taozhiyy.top";

export function shouldRequireOwnerLogin({ hostname = "" } = {}) {
  return hostname === PWA_AUTH_HOSTNAME;
}

export function shouldOpenAppConsoleAtRoot({ hostname = "", pathname = "" } = {}) {
  return shouldRequireOwnerLogin({ hostname }) && pathname === "/";
}

// /app 在所有主机都可见：未登录时 AppConsolePage 只显示登录卡，门禁在登录表单，
// 敏感接口另有会话鉴权。app.taozhiyy.top 的强制登录行为由 shouldRequireOwnerLogin 管。
export function shouldExposeAppConsole() {
  return true;
}

export function userHasOwnerAppAccess(auth) {
  return Boolean(auth?.loggedIn && auth?.unlimited && auth?.user?.isOwner);
}

export function getAppAccessState({ hostname = "", auth, isLoading = false } = {}) {
  if (!shouldRequireOwnerLogin({ hostname })) return "allowed";
  if (userHasOwnerAppAccess(auth)) return "allowed";
  if (isLoading) return "loading";
  return "blocked";
}

export function shouldShowOwnerLoginActions(accessState) {
  return accessState === "blocked";
}
