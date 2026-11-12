import test from "node:test";
import assert from "node:assert/strict";

import {
  appendReplyToFriendsThreads,
  findFriendsThreadRootId,
  flattenGuestbookThreads,
  flattenThreadReplies,
  isFriendApplicationContent,
  normalizeFriendsThreads,
} from "./friendsApplicationThreads.js";

test("detects valid friend application content", () => {
  assert.equal(
    isFriendApplicationContent(
      "站点名称：桃之夭夭\n站点链接：https://taozhiyy.top\n站点描述：桃之夭夭的小屋\n头像链接：https://img.test/1.png",
    ),
    true,
  );
});

test("keeps freeform top-level items and their replies", () => {
  const rows = [
    {
      id: 1,
      parentId: 0,
      content:
        "站点名称：桃之夭夭\n站点链接：https://taozhiyy.top\n站点描述：桃之夭夭的小屋\n头像链接：https://img.test/1.png",
      replies: [{ id: 2, parentId: 1, content: "已回访" }],
      replyCount: 1,
    },
    { id: 3, parentId: 0, content: "普通留言", replies: [], replyCount: 0 },
  ];

  const threads = normalizeFriendsThreads(rows);

  assert.equal(threads.length, 2);
  assert.equal(threads[0].replies.length, 1);
  assert.equal(threads[0].replyCount, 1);
  assert.equal(threads[1].id, 3);
});

test("flattens threaded guestbook rows for generic guestbook page usage", () => {
  const rows = [
    {
      id: 1,
      parentId: 0,
      content: "顶层",
      replies: [{ id: 2, parentId: 1, content: "回复" }],
    },
  ];

  const flattened = flattenGuestbookThreads(rows);

  assert.deepEqual(
    flattened.map((item) => item.id),
    [1, 2],
  );
});

test("flattens multi-level friend replies for timeline rendering", () => {
  const flattened = flattenThreadReplies([
    {
      id: 2,
      parentId: 1,
      content: "first",
      replies: [{ id: 3, parentId: 2, content: "second" }],
    },
  ]);

  assert.deepEqual(
    flattened.map((item) => [item.id, item.depth]),
    [
      [2, 1],
      [3, 2],
    ],
  );
});

test("appends a reply under any existing friend thread message", () => {
  const threads = [
    {
      id: 1,
      parentId: 0,
      content: "root",
      replyCount: 1,
      replies: [{ id: 2, parentId: 1, content: "first", replies: [] }],
    },
  ];

  const next = appendReplyToFriendsThreads(threads, 2, {
    id: 3,
    parentId: 2,
    content: "second",
  });

  assert.equal(next[0].replyCount, 2);
  assert.deepEqual(
    flattenThreadReplies(next[0].replies).map((item) => item.id),
    [2, 3],
  );
});

test("finds the root thread for nested friend replies", () => {
  const threads = [
    {
      id: 1,
      parentId: 0,
      content: "root",
      replies: [
        {
          id: 2,
          parentId: 1,
          content: "first",
          replies: [{ id: 3, parentId: 2 }],
        },
      ],
    },
  ];

  assert.equal(findFriendsThreadRootId(threads, 3), 1);
  assert.equal(findFriendsThreadRootId(threads, 99), null);
});
