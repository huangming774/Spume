import test from "node:test";
import assert from "node:assert/strict";

import { getRadarFeed } from "./acgApi.js";

const createJsonResponse = (payload) => ({
  ok: true,
  headers: {
    get(name) {
      return name.toLowerCase() === "content-type"
        ? "application/json; charset=utf-8"
        : null;
    },
  },
  async json() {
    return payload;
  },
});

test("getRadarFeed keeps real creator items from the API", async (t) => {
  const originalFetch = global.fetch;
  t.after(() => {
    global.fetch = originalFetch;
  });

  global.fetch = async () =>
    createJsonResponse({
      items: [
        {
          id: "r_real",
          creatorName: "UP · 桃之夭夭",
          latestText: "新视频发布了",
        },
      ],
    });

  const items = await getRadarFeed();

  assert.equal(items.length, 1);
  assert.equal(items[0].creatorName, "UP · 桃之夭夭");
});

test("getRadarFeed returns an empty list when radar sync is unavailable", async (t) => {
  const originalFetch = global.fetch;
  t.after(() => {
    global.fetch = originalFetch;
  });

  global.fetch = async () => {
    throw new Error("radar unavailable");
  };

  const items = await getRadarFeed();

  assert.deepEqual(items, []);
});
