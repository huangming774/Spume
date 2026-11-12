/** Normalize API payloads to a safe array for .map(). */
export function asList(data, fallback = []) {
  if (Array.isArray(data)) return data;
  if (Array.isArray(data?.items)) return data.items;
  return fallback;
}
