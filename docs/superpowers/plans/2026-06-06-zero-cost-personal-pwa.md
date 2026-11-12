# Zero-Cost Personal PWA Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add zero-cost PWA install support to the existing `main/` React site for personal desktop and mobile use.

**Architecture:** Keep the current React/Vite app and Go API untouched. Add static PWA assets under `main/public/`, link the manifest from `main/index.html`, and register a conservative production-only service worker from `main/src/main.jsx`.

**Tech Stack:** React 18, Vite, browser Web App Manifest, browser Service Worker API, Node built-in test runner.

---

## File Structure

- Create `main/src/pwa/registerServiceWorker.js`: owns feature detection and service-worker registration.
- Create `main/src/pwa/registerServiceWorker.test.js`: verifies registration decisions with Node tests.
- Create `main/public/manifest.webmanifest`: describes the installable personal app.
- Create `main/public/sw.js`: caches the app shell and same-origin static assets conservatively.
- Modify `main/index.html`: adds manifest and install-related metadata.
- Modify `main/src/main.jsx`: calls the registration module after rendering.

## Task 1: Service Worker Registration Logic

**Files:**
- Create: `main/src/pwa/registerServiceWorker.test.js`
- Create: `main/src/pwa/registerServiceWorker.js`
- Modify: `main/src/main.jsx`

- [ ] **Step 1: Write the failing test**

```js
import test from "node:test";
import assert from "node:assert/strict";

import {
  getServiceWorkerUrl,
  shouldRegisterServiceWorker,
} from "./registerServiceWorker.js";

test("does not register outside production builds", () => {
  assert.equal(
    shouldRegisterServiceWorker({
      isProduction: false,
      navigatorRef: { serviceWorker: {} },
      windowRef: { location: { protocol: "https:" } },
    }),
    false,
  );
});

test("registers only when service workers are available on a safe protocol", () => {
  assert.equal(
    shouldRegisterServiceWorker({
      isProduction: true,
      navigatorRef: { serviceWorker: {} },
      windowRef: { location: { protocol: "https:" } },
    }),
    true,
  );

  assert.equal(
    shouldRegisterServiceWorker({
      isProduction: true,
      navigatorRef: {},
      windowRef: { location: { protocol: "https:" } },
    }),
    false,
  );
});

test("allows localhost registration for preview builds", () => {
  assert.equal(
    shouldRegisterServiceWorker({
      isProduction: true,
      navigatorRef: { serviceWorker: {} },
      windowRef: {
        location: { protocol: "http:", hostname: "localhost" },
      },
    }),
    true,
  );
});

test("uses the root service worker URL", () => {
  assert.equal(getServiceWorkerUrl(), "/sw.js");
});
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `node --test main/src/pwa/registerServiceWorker.test.js`

Expected: FAIL because `main/src/pwa/registerServiceWorker.js` does not exist yet.

- [ ] **Step 3: Write the minimal implementation**

```js
export function getServiceWorkerUrl() {
  return "/sw.js";
}

function isLocalhost(hostname = "") {
  return hostname === "localhost" || hostname === "127.0.0.1" || hostname === "::1";
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

  return location.protocol === "https:" || isLocalhost(location.hostname);
}

export function registerServiceWorker({
  isProduction = import.meta.env.PROD,
  navigatorRef = globalThis.navigator,
  windowRef = globalThis.window,
} = {}) {
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
```

Then modify `main/src/main.jsx`:

```js
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App.jsx";
import { registerServiceWorker } from "./pwa/registerServiceWorker.js";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <App />
  </StrictMode>,
);

registerServiceWorker();
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `node --test main/src/pwa/registerServiceWorker.test.js`

Expected: PASS.

## Task 2: Static PWA Assets

**Files:**
- Create: `main/public/manifest.webmanifest`
- Create: `main/public/sw.js`
- Modify: `main/index.html`

- [ ] **Step 1: Add the manifest**

Create `main/public/manifest.webmanifest`:

```json
{
  "name": "桃之夭夭",
  "short_name": "桃之夭夭",
  "description": "A personal site app for 桃之夭夭.",
  "start_url": "/",
  "scope": "/",
  "display": "standalone",
  "background_color": "#0b1020",
  "theme_color": "#f472b6",
  "orientation": "portrait-primary",
  "icons": [
    {
      "src": "/img/logo.png",
      "sizes": "512x512",
      "type": "image/png",
      "purpose": "any maskable"
    }
  ],
  "shortcuts": [
    {
      "name": "留言板",
      "short_name": "留言",
      "url": "/guestbook",
      "description": "Open the guestbook"
    },
    {
      "name": "Bili Hub",
      "short_name": "Bili",
      "url": "/bili",
      "description": "Open Bili Hub"
    }
  ]
}
```

- [ ] **Step 2: Add the service worker**

Create `main/public/sw.js`:

```js
const CACHE_NAME = "taozhiyy-pwa-v1";
const APP_SHELL = ["/", "/manifest.webmanifest", "/img/logo.png"];

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

  if (request.mode === "navigate") {
    event.respondWith(
      fetch(request).catch(() => caches.match("/")),
    );
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
```

- [ ] **Step 3: Link PWA metadata in HTML**

Update `main/index.html` head:

```html
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <link rel="manifest" href="/manifest.webmanifest" />
    <meta name="theme-color" content="#f472b6" />
    <meta name="mobile-web-app-capable" content="yes" />
    <meta name="apple-mobile-web-app-capable" content="yes" />
    <meta name="apple-mobile-web-app-title" content="桃之夭夭" />
```

Keep the existing favicon, viewport, title, preload, assistant scripts, and React entry.

## Task 3: Verification

**Files:**
- Verify generated output under `main/dist/`

- [ ] **Step 1: Run focused test**

Run: `node --test main/src/pwa/registerServiceWorker.test.js`

Expected: PASS.

- [ ] **Step 2: Run production build**

Run from `main/`: `npm run build`

Expected: Vite build completes and `dist/manifest.webmanifest` plus `dist/sw.js` exist.

- [ ] **Step 3: Inspect git diff**

Run: `git status --short` and `git diff --stat`

Expected: only the PWA plan, registration module/test, manifest, service worker, `index.html`, and `main.jsx` are changed.
