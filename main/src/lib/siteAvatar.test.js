import test from "node:test";
import assert from "node:assert/strict";

import { defaultSiteAvatarUrl, siteAvatarFromProfile } from "./siteAvatar.js";

test("uses a stored public avatar and otherwise keeps the published default", () => {
  assert.equal(
    siteAvatarFromProfile({ avatar: "https://fs.964643.xyz/avatars/face.png" }),
    "https://fs.964643.xyz/avatars/face.png",
  );
  assert.equal(siteAvatarFromProfile({ avatar: "" }), defaultSiteAvatarUrl);
  assert.equal(siteAvatarFromProfile({ avatar: "javascript:alert(1)" }), defaultSiteAvatarUrl);
  assert.equal(siteAvatarFromProfile(null), defaultSiteAvatarUrl);
  assert.match(defaultSiteAvatarUrl, /^https:\/\/fs\.964643\.xyz\/1\.png$/);
});
