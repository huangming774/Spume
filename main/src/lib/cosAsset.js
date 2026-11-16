// "cosAsset" is a historical name: these files used to live in the Tencent COS
// bucket and were served through the /cos/ same-origin proxy. The objects now
// live in the Cloudflare R2 "rustblog" bucket and are linked directly at
// https://fs.964643.xyz/<key> (object keys unchanged).
const R2_ASSET_ORIGIN = "https://fs.964643.xyz";

export const cosAsset = (path) => {
  const key = String(path).replace(/^\/+/, "");
  // Callers join results as `${cosAsset(...)}/file`, so the empty path must not
  // carry a trailing slash (R2 addresses keys with a single leading slash).
  return key ? `${R2_ASSET_ORIGIN}/${key}` : R2_ASSET_ORIGIN;
};
