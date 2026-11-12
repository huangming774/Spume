import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const pwaDir = dirname(fileURLToPath(import.meta.url));
const mainDir = resolve(pwaDir, "..", "..");
const publicDir = resolve(mainDir, "public");

test("root HTML does not expose PWA install metadata before host gating", async () => {
  const html = await readFile(resolve(mainDir, "index.html"), "utf8");

  assert.doesNotMatch(html, /rel="manifest"/);
  assert.doesNotMatch(html, /name="theme-color"/);
  assert.doesNotMatch(html, /apple-mobile-web-app/);
});

test("manifest describes an installable personal app", async () => {
  const manifestPath = resolve(publicDir, "manifest.webmanifest");
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));

  assert.equal(manifest.name, "桃之夭夭");
  assert.equal(manifest.start_url, "/app");
  assert.equal(manifest.scope, "/");
  assert.equal(manifest.display, "standalone");
  assert.equal(manifest.background_color, "#fffaf2");
  assert.equal(manifest.theme_color, "#fffaf2");
  assert.ok(
    manifest.icons.some(
      (icon) =>
        icon.src === "/pwa-icon-192.png" &&
        icon.sizes === "192x192" &&
        icon.type === "image/png" &&
        icon.purpose === "any",
    ),
  );
  assert.ok(
    manifest.icons.some(
      (icon) =>
        icon.src === "/pwa-icon-512.png" &&
        icon.sizes === "512x512" &&
        icon.type === "image/png" &&
        icon.purpose === "any maskable",
    ),
  );
});

test("PWA install metadata uses the site white chrome color", async () => {
  const source = await readFile(resolve(mainDir, "src", "pwa", "registerServiceWorker.js"), "utf8");

  assert.match(source, /setMetaTag\(documentRef,\s*"theme-color",\s*"#fffaf2"\)/);
  assert.doesNotMatch(source, /#f472b6/);
});

test("service worker leaves live API traffic online-first", async () => {
  const workerPath = resolve(publicDir, "sw.js");
  const workerSource = await readFile(workerPath, "utf8");

  assert.match(workerSource, /url\.pathname\.startsWith\("\/api\/"\)/);
  assert.match(workerSource, /return;/);
});

test("service worker does not stale-cache deployed app assets or about media", async () => {
  const workerPath = resolve(publicDir, "sw.js");
  const workerSource = await readFile(workerPath, "utf8");

  assert.match(workerSource, /url\.pathname\.startsWith\("\/assets\/"\)/);
  assert.match(workerSource, /url\.pathname\.startsWith\("\/cos\/"\)/);
  assert.match(workerSource, /url\.pathname === "\/about-preview\.html"/);
});

test("service worker does not stale-cache deployed showcase pages", async () => {
  const workerPath = resolve(publicDir, "sw.js");
  const workerSource = await readFile(workerPath, "utf8");

  assert.match(workerSource, /url\.pathname\.startsWith\("\/showcase\/"\)/);
  assert.match(workerSource, /url\.pathname\.startsWith\("\/web\/"\)/);
});

test("service worker uses the app console as the install shell", async () => {
  const workerPath = resolve(publicDir, "sw.js");
  const workerSource = await readFile(workerPath, "utf8");

  assert.match(workerSource, /const APP_START_URL = "\/app";/);
  assert.match(workerSource, /caches\.match\(APP_START_URL\)/);
  assert.doesNotMatch(workerSource, /const APP_SHELL = \["\/",/);
});
