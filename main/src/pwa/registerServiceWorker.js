export const APP_PWA_HOSTNAME = "app.taozhiyy.top";

export function getServiceWorkerUrl() {
  return "/sw.js";
}

function isLocalhost(hostname = "") {
  return hostname === "localhost" || hostname === "127.0.0.1" || hostname === "::1";
}

export function shouldExposePwaInstallMetadata({ windowRef } = {}) {
  const hostname = windowRef?.location?.hostname || "";
  return hostname === APP_PWA_HOSTNAME || isLocalhost(hostname);
}

export function shouldRegisterServiceWorker({
  isProduction,
  navigatorRef,
  windowRef,
} = {}) {
  if (!isProduction) return false;
  if (!navigatorRef || !("serviceWorker" in navigatorRef)) return false;

  const location = windowRef?.location;
  if (!location) return false;
  if (!shouldExposePwaInstallMetadata({ windowRef })) return false;

  return location.protocol === "https:" || isLocalhost(location.hostname);
}

export function shouldUnregisterServiceWorkers({ navigatorRef, windowRef } = {}) {
  if (!navigatorRef || !("serviceWorker" in navigatorRef)) return false;
  return !shouldExposePwaInstallMetadata({ windowRef });
}

function setMetaTag(documentRef, name, content) {
  const selector = `meta[name="${name}"]`;
  const meta = documentRef.querySelector(selector) || documentRef.createElement("meta");
  meta.setAttribute("name", name);
  meta.setAttribute("content", content);
  if (!meta.parentNode) {
    documentRef.head.appendChild(meta);
  }
}

export function applyPwaInstallMetadata({
  documentRef = globalThis.document,
  windowRef = globalThis.window,
} = {}) {
  if (!documentRef || !shouldExposePwaInstallMetadata({ windowRef })) return;

  if (!documentRef.querySelector('link[rel="manifest"]')) {
    const manifest = documentRef.createElement("link");
    manifest.setAttribute("rel", "manifest");
    manifest.setAttribute("href", "/manifest.webmanifest");
    documentRef.head.appendChild(manifest);
  }

  setMetaTag(documentRef, "theme-color", "#fffaf2");
  setMetaTag(documentRef, "mobile-web-app-capable", "yes");
  setMetaTag(documentRef, "apple-mobile-web-app-capable", "yes");
  setMetaTag(documentRef, "apple-mobile-web-app-title", "桃之夭夭");
}

export function registerServiceWorker({
  isProduction = Boolean(import.meta.env?.PROD),
  documentRef = globalThis.document,
  navigatorRef = globalThis.navigator,
  windowRef = globalThis.window,
} = {}) {
  applyPwaInstallMetadata({ documentRef, windowRef });

  if (shouldUnregisterServiceWorkers({ navigatorRef, windowRef })) {
    navigatorRef.serviceWorker
      .getRegistrations()
      .then((registrations) => registrations.forEach((registration) => registration.unregister()))
      .catch(() => {});
    return;
  }

  if (
    !shouldRegisterServiceWorker({
      isProduction,
      navigatorRef,
      windowRef,
    })
  ) {
    return;
  }

  windowRef.addEventListener("load", () => {
    navigatorRef.serviceWorker.register(getServiceWorkerUrl()).catch(() => {});
  });
}
