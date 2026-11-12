# Zero-Cost Personal PWA Design

## Goal

Make the existing `main/` React site installable and convenient to use on desktop and mobile at zero cost, without app-store publishing, paid certificates, or separate native app maintenance.

## Scope

This design adds a Progressive Web App layer to the current website. The app remains the same React/Vite frontend connected to the existing `acg-api` backend through the current `/api` routes. The first version targets personal use through browser installation:

- Desktop: install from Chrome or Edge.
- Android: add to home screen or install from a supporting browser.
- iPhone: add to home screen from Safari.

No Tauri, Electron, Capacitor, app-store upload, paid signing, or paid developer account is included in this first version.

## Architecture

The PWA layer is made of three small pieces:

- A web app manifest in `main/public/manifest.webmanifest` that describes the app name, icons, start URL, display mode, colors, and shortcuts.
- A service worker in `main/public/sw.js` that precaches the app shell and uses conservative runtime caching for same-origin assets.
- A focused registration module in `main/src/pwa/registerServiceWorker.js` that registers the service worker only in production-like browser contexts where service workers are available.

The React application keeps using `BrowserRouter` and existing routes. API calls remain online-first because guestbook, login, AI, and Bili data should come from the live backend.

## User Experience

The site opens like a normal app after installation. It keeps the current visual identity and avoids adding visible setup instructions inside the UI. Browser install prompts and home-screen behavior are handled by the platform.

Offline behavior is intentionally modest: the app shell can load when cached, but live API features may show their existing empty, fallback, or failed states when the network is unavailable.

## Testing

Add Node tests for the registration decision logic so service-worker registration does not run in unsupported environments and points to `/sw.js` when enabled. Build verification should run `npm run build` in `main/` to confirm the manifest and service worker are copied into `dist/`.

## Open Source Notes

The implementation stays simple for GitHub readers: no secret files, no store credentials, no generated native folders, and no paid platform assumptions. Documentation should explain that this is a zero-cost personal PWA path and that native packaging can be added later if needed.
