export function getOwnerSessionLabel(auth) {
  if (!auth?.loggedIn) return "站长未确认";
  if (auth?.user?.isOwner && !auth?.unlimited) return "站长安全验证待完成";
  return auth?.user?.displayName || auth?.user?.email || "站长";
}

export function getBackendHealthLabel(health) {
  return health?.status || "检查中";
}

export function getStatsSnapshot(stats) {
  const summary = stats?.summary || {};
  return {
    configured: Boolean(stats?.configured),
    model: stats?.model || "未知模型",
    today: `${summary.todaySuccess ?? 0} / ${summary.todayTotal ?? 0}`,
    period: `${summary.periodSuccess ?? 0} / ${summary.periodTotal ?? 0}`,
    successRate: summary.successRateText || "0%",
  };
}

export function getOwnerRegisteredUsers(ownerEmails) {
  const items = ownerEmails?.registeredUsers;
  if (!Array.isArray(items)) return [];
  return items
    .filter((item) => item?.email)
    .map((item) => ({
      id: item.id,
      email: item.email,
      displayName: item.displayName || "",
      createdAt: item.createdAt || "",
    }));
}

export function getOwnerGuestbookContacts(ownerEmails) {
  const items = ownerEmails?.guestbookContacts;
  if (!Array.isArray(items)) return [];
  return items
    .filter((item) => item?.contactEmail)
    .map((item) => ({
      id: item.id,
      source: item.source || "",
      nickname: item.nickname || "",
      content: item.content || "",
      contactEmail: item.contactEmail,
      accountEmail: item.accountEmail || "",
      createdAt: item.createdAt || "",
    }));
}

export function getOwnerFixedAnswers(ownerStatus) {
  const items = ownerStatus?.ai?.fixedAnswers;
  if (!Array.isArray(items)) return [];
  return items
    .filter((item) => item?.question && item?.answer)
    .map((item) => ({
      id: item.id,
      question: item.question,
      answer: item.answer,
      status: item.status || "active",
      createdAt: item.createdAt || "",
      updatedAt: item.updatedAt || "",
    }));
}
