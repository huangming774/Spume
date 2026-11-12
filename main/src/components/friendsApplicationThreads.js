export const FRIEND_APPLICATION_TEMPLATE_FIELDS = [
  "站点名称",
  "站点链接",
  "站点描述",
  "头像链接",
];

export function isFriendApplicationContent(content = "") {
  return FRIEND_APPLICATION_TEMPLATE_FIELDS.every((field) =>
    content.includes(field),
  );
}

export function normalizeFriendsThreads(items = []) {
  return items
    .filter((item) => item.parentId === 0)
    .map((item) => normalizeFriendThreadNode(item));
}

export function flattenGuestbookThreads(items = []) {
  return items.flatMap((item) => {
    const replies = flattenThreadReplies(item.replies);
    if (replies.length === 0) return [item];
    return [item, ...replies];
  });
}

export function flattenThreadReplies(replies = [], depth = 1) {
  return replies.flatMap((reply) => {
    const normalized = normalizeFriendThreadNode(reply);
    return [
      { ...normalized, depth },
      ...flattenThreadReplies(normalized.replies, depth + 1),
    ];
  });
}

export function appendReplyToFriendsThreads(threads = [], parentId, reply) {
  return threads.map((thread) =>
    appendReplyToFriendThreadNode(thread, parentId, reply),
  );
}

export function findFriendsThreadRootId(threads = [], messageId) {
  const targetId = Number(messageId);
  for (const thread of threads) {
    if (threadContainsMessage(thread, targetId)) {
      return thread.id;
    }
  }
  return null;
}

function normalizeFriendThreadNode(item = {}) {
  const replies = Array.isArray(item.replies)
    ? item.replies.map((reply) => normalizeFriendThreadNode(reply))
    : [];
  return {
    ...item,
    replies,
    replyCount:
      typeof item.replyCount === "number"
        ? item.replyCount
        : replies.reduce(
            (total, reply) => total + 1 + (reply.replyCount || 0),
            0,
          ),
  };
}

function appendReplyToFriendThreadNode(node, parentId, reply) {
  const replies = Array.isArray(node.replies) ? node.replies : [];
  if (node.id === parentId) {
    return {
      ...node,
      replies: [...replies, normalizeFriendThreadNode(reply)],
      replyCount: (node.replyCount || 0) + 1,
    };
  }

  let changed = false;
  const nextReplies = replies.map((child) => {
    const next = appendReplyToFriendThreadNode(child, parentId, reply);
    if (next !== child) {
      changed = true;
    }
    return next;
  });

  if (!changed) {
    return node;
  }
  return {
    ...node,
    replies: nextReplies,
    replyCount: (node.replyCount || 0) + 1,
  };
}

function threadContainsMessage(node, messageId) {
  if (node.id === messageId) {
    return true;
  }
  const replies = Array.isArray(node.replies) ? node.replies : [];
  return replies.some((reply) => threadContainsMessage(reply, messageId));
}
