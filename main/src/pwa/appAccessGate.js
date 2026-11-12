export const PWA_AUTH_HOSTNAME = "app.taozhiyy.top";

function isLocalhost(hostname = "") {
  return hostname === "localhost" || hostname === "127.0.0.1" || hostname === "::1";
}

export function shouldRequireOwnerLogin({ hostname = "" } = {}) {
  return hostname === PWA_AUTH_HOSTNAME;
}

export function shouldOpenAppConsoleAtRoot({ hostname = "", pathname = "" } = {}) {
  return shouldRequireOwnerLogin({ hostname }) && pathname === "/";
}

export function shouldExposeAppConsole({ hostname = "" } = {}) {
  return shouldRequireOwnerLogin({ hostname }) || isLocalhost(hostname);
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
