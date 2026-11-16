import test from "node:test";
import assert from "node:assert/strict";

import { authUploadAvatar } from "./authApi.js";

test("authUploadAvatar posts the image to the account avatar endpoint", async (t) => {
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
          ok: true,
          user: {
            id: 1,
            avatar: "https://fs.964643.xyz/avatars/face.png",
          },
        };
      },
    };
  };

  const file = new File([Uint8Array.from([137, 80, 78, 71])], "face.png", {
    type: "image/png",
  });
  const result = await authUploadAvatar(file);

  assert.equal(requestURL, "/api/auth/avatar");
  assert.equal(requestOptions.method, "POST");
  assert.equal(requestOptions.credentials, "include");
  assert.equal(requestOptions.headers.Accept, "application/json");
  assert.equal(requestOptions.headers["Content-Type"], undefined);
  assert.equal(requestOptions.body instanceof FormData, true);
  assert.equal(requestOptions.body.get("file"), file);
  assert.equal(result.ok, true);
  assert.equal(result.data.user.avatar, "https://fs.964643.xyz/avatars/face.png");
});
