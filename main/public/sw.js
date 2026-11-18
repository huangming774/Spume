const CACHE_NAME = "taozhiyy-pwa-v6";
const APP_START_URL = "/app";
const APP_SHELL = [APP_START_URL, "/manifest.webmanifest", "/pwa-icon-192.png", "/pwa-icon-512.png"];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(APP_SHELL)),
  );
  self.skipWaiting();
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key !== CACHE_NAME)
            .map((key) => caches.delete(key)),
        ),
      ),
  );
  self.clients.claim();
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET") return;

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;
  if (url.pathname.startsWith("/api/")) return;
  if (url.pathname.startsWith("/assets/")) return;
  if (url.pathname.startsWith("/projects/")) return;
  if (url.pathname.startsWith("/cos/")) return;
  // 小精灵的 CSS/JS 在 shared/ 里随时会改，缓存了就再也拿不到新版（滚动条/布局会停留在旧样式）。
  if (url.pathname.startsWith("/ai-assistant/")) return;
  // 看板娘 widget 与模型同理，走网络不缓存，换模型/改文案即时生效。
  if (url.pathname.startsWith("/live2d/")) return;
  if (url.pathname === "/about-preview.html") return;
  if (url.pathname.startsWith("/showcase/") || url.pathname.startsWith("/web/")) return;

  if (request.mode === "navigate") {
    event.respondWith(fetch(request).catch(() => caches.match(APP_START_URL)));
    return;
  }

  event.respondWith(
    caches.match(request).then((cached) => {
      if (cached) return cached;

      return fetch(request).then((response) => {
        if (!response || response.status !== 200 || response.type !== "basic") {
          return response;
        }

        const copy = response.clone();
        caches.open(CACHE_NAME).then((cache) => cache.put(request, copy));
        return response;
      });
    }),
  );
});
