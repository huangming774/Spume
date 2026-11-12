export function journalTo(path = "/") {
  if (!path || path === "/") return "/build";
  return `/build${path.startsWith("/") ? path : `/${path}`}`;
}
