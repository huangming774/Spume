export const COS_PROXY_PREFIX = "/cos";

export const cosAsset = (path) =>
  `${COS_PROXY_PREFIX}/${String(path).replace(/^\/+/, "")}`;
