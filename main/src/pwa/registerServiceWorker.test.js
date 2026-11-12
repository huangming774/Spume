import test from "node:test";
import assert from "node:assert/strict";

import {
  APP_PWA_HOSTNAME,
  getServiceWorkerUrl,
  shouldExposePwaInstallMetadata,
  shouldRegisterServiceWorker,
  shouldUnregisterServiceWorkers,
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
      windowRef: { location: { protocol: "https:", hostname: APP_PWA_HOSTNAME } },
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

test("does not register on the public root domain", () => {
  assert.equal(
    shouldRegisterServiceWorker({
      isProduction: true,
      navigatorRef: { serviceWorker: {} },
      windowRef: { location: { protocol: "https:", hostname: "taozhiyy.top" } },
    }),
    false,
  );
});

test("exposes install metadata only on the app subdomain and local preview", () => {
  assert.equal(
    shouldExposePwaInstallMetadata({
      windowRef: { location: { hostname: APP_PWA_HOSTNAME } },
    }),
    true,
  );
  assert.equal(
    shouldExposePwaInstallMetadata({
      windowRef: { location: { hostname: "localhost" } },
    }),
    true,
  );
  assert.equal(
    shouldExposePwaInstallMetadata({
      windowRef: { location: { hostname: "taozhiyy.top" } },
    }),
    false,
  );
});

test("removes existing service workers on the public root domain only", () => {
  assert.equal(
    shouldUnregisterServiceWorkers({
      navigatorRef: { serviceWorker: {} },
      windowRef: { location: { hostname: "taozhiyy.top" } },
    }),
    true,
  );
  assert.equal(
    shouldUnregisterServiceWorkers({
      navigatorRef: { serviceWorker: {} },
      windowRef: { location: { hostname: APP_PWA_HOSTNAME } },
    }),
    false,
  );
  assert.equal(
    shouldUnregisterServiceWorkers({
      navigatorRef: {},
      windowRef: { location: { hostname: "taozhiyy.top" } },
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
