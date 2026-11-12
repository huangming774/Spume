const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export function normalizeContactEmail(value = "") {
  return value.trim().toLowerCase();
}

export function buildFriendsApplicationPayload({ nickname, contactEmail, content }) {
  const body = content.trim();
  const email = normalizeContactEmail(contactEmail);
  const name = nickname.trim();

  if (!body) {
    return { ok: false, error: "请先写下留言内容。" };
  }

  if (!name) {
    return { ok: false, error: "请留下昵称。" };
  }

  if (!email) {
    return { ok: false, error: "请留下邮箱，方便收到回复通知。" };
  }

  if (email && !emailPattern.test(email)) {
    return { ok: false, error: "请输入有效的邮箱地址。" };
  }

  return {
    ok: true,
    payload: {
      nickname: name,
      contactEmail: email,
      content: body,
    },
  };
}
