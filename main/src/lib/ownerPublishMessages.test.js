import test from "node:test";
import assert from "node:assert/strict";

import { ownerFriendPublishToast } from "./ownerPublishMessages.js";

test("ownerFriendPublishToast tells owner to merge the created friend PR", () => {
  const toast = ownerFriendPublishToast({
    path: "main/src/data/friendCards.js",
    commitSha: "friend-commit-sha",
    branch: "owner/friend-20260813-120000-example",
    pullRequestURL: "https://github.com/octo/taozhiyy/pull/42",
    pullRequestNumber: 42,
  });

  assert.match(toast, /Pull Request #42/);
  assert.match(toast, /https:\/\/github\.com\/octo\/taozhiyy\/pull\/42/);
  assert.match(toast, /合并后服务器会从 master 自动部署/);
});

test("ownerFriendPublishToast keeps duplicate friend publish as no-op", () => {
  assert.equal(
    ownerFriendPublishToast({ path: "main/src/data/friendCards.js", changed: false }),
    "这条友链已经存在于 main/src/data/friendCards.js。",
  );
});
