import test from "node:test";
import assert from "node:assert/strict";

import { getGuestbook, postGuestbook } from "./guestbookApi.js";

test("getGuestbook reads the real guestbook messages endpoint", async (t) => {
  const originalFetch = globalThis.fetch;
  t.after(() => {
    globalThis.fetch = originalFetch;
  });

  let requestURL = "";
  let requestOptions = null;
  globalThis.fetch = async (url, options = {}) => {
    requestURL = url;
    requestOptions = options;
    return {
      ok: true,
      async json() {
        return {
          items: [
            {
              id: 7,
              nickname: "桃之夭夭",
              content: "真实留言",
              createdAt: "2026-06-07 18:00",
            },
          ],
        };
      },
    };
  };

  const rows = await getGuestbook(50);

  assert.equal(requestURL, "/api/guestbook/messages?page=1&pageSize=50&channel=guestbook");
  assert.equal(requestOptions.credentials, "include");
  assert.equal(requestOptions.headers.Accept, "application/json");
  assert.deepEqual(rows, [
    {
      id: 7,
      name: "桃之夭夭",
      content: "真实留言",
      createdAt: "2026-06-07 18:00",
    },
  ]);
});

test("getGuestbook reports backend failures instead of returning mock entries", async (t) => {
  const originalFetch = globalThis.fetch;
  t.after(() => {
    globalThis.fetch = originalFetch;
  });

  globalThis.fetch = async () => ({
    ok: false,
    status: 503,
    async json() {
      return { message: "留言后端暂时不可用" };
    },
  });

  await assert.rejects(() => getGuestbook(50), /留言后端暂时不可用/);
});

test("postGuestbook creates a real guestbook message and never fabricates offline rows", async (t) => {
  const originalFetch = globalThis.fetch;
  t.after(() => {
    globalThis.fetch = originalFetch;
  });

  let requestURL = "";
  let requestOptions = null;
  globalThis.fetch = async (url, options = {}) => {
    requestURL = url;
    requestOptions = options;
    return {
      ok: true,
      async json() {
        return {
          item: {
            id: 8,
            nickname: "桃之夭夭",
            content: "后端写入",
            createdAt: "2026-06-07 18:01",
          },
        };
      },
    };
  };

  const row = await postGuestbook({ name: " 桃之夭夭 ", content: " 后端写入 " });

  assert.equal(requestURL, "/api/guestbook/messages");
  assert.equal(requestOptions.method, "POST");
  assert.equal(requestOptions.credentials, "include");
  assert.equal(requestOptions.headers["Content-Type"], "application/json");
  assert.deepEqual(JSON.parse(requestOptions.body), {
    nickname: "桃之夭夭",
    content: "后端写入",
    channel: "guestbook",
  });
  assert.deepEqual(row, {
    id: 8,
    name: "桃之夭夭",
    content: "后端写入",
    createdAt: "2026-06-07 18:01",
  });
});

test("postGuestbook reports backend failures instead of creating offline rows", async (t) => {
  const originalFetch = globalThis.fetch;
  t.after(() => {
    globalThis.fetch = originalFetch;
  });

  globalThis.fetch = async () => ({
    ok: false,
    status: 429,
    async json() {
      return { message: "留言太频繁啦" };
    },
  });

  await assert.rejects(
    () => postGuestbook({ name: "桃之夭夭", content: "再试一次" }),
    /留言太频繁啦/,
  );
});
