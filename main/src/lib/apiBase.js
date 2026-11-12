/** API root: set VITE_API_BASE in production (e.g. https://taozhiyy.top). */
export const API_BASE = (import.meta.env?.VITE_API_BASE || "").replace(/\/$/, "");

export const apiUrl = (path) => {
  const p = path.startsWith("/") ? path : `/${path}`;
  return API_BASE ? `${API_BASE}${p}` : p;
};
