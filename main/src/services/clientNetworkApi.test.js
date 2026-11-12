import test from "node:test";
import assert from "node:assert/strict";

import { measureClientNetwork } from "./clientNetworkApi.js";

test("measureClientNetwork returns compact visitor network data and request latency", async () => {
  let requestedUrl = "";
  const result = await measureClientNetwork({
    fetchImpl: async (url) => {
      requestedUrl = url;
      return {
        ok: true,
        json: async () => ({
          regionLabel: "杭州",
          ipMasked: "123.45.*.*",
          serverTime: "2026-06-26T00:00:00Z",
        }),
      };
    },
    now: (() => {
      const marks = [100, 138];
      return () => marks.shift();
    })(),
  });

  assert.match(requestedUrl, /^\/api\/client\/network\?ts=\d+/);
  assert.deepEqual(result, {
    addressLabel: "杭州",
    regionLabel: "杭州",
    ipMasked: "123.45.*.*",
    serverTime: "2026-06-26T00:00:00Z",
    latencyMs: 38,
  });
});

test("measureClientNetwork falls back to masked IP when region is generic visitor label", async () => {
  const result = await measureClientNetwork({
    fetchImpl: async () => ({
      ok: true,
      json: async () => ({
        regionLabel: "访客",
        ipMasked: "203.0.*.*",
      }),
    }),
    now: (() => {
      const marks = [10, 31];
      return () => marks.shift();
    })(),
  });

  assert.equal(result.addressLabel, "203.0.*.*");
  assert.equal(result.latencyMs, 21);
});

test("measureClientNetwork throws readable errors for failed responses", async () => {
  await assert.rejects(
    () =>
      measureClientNetwork({
        fetchImpl: async () => ({
          ok: false,
          json: async () => ({ message: "network unavailable" }),
        }),
        now: () => 0,
      }),
    /network unavailable/,
  );
});
