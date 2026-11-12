# Friends Page Threaded Replies Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Upgrade `/friends` into the new three-section poetic page and add real persisted replies to the reused guestbook message flow so friend-link applications can be discussed in-thread.

**Architecture:** Extend `guestbook_messages` with a lightweight parent-child relationship instead of creating a separate replies table. Keep the API path stable by making reply support additive on `GET /api/guestbook/messages` and `POST /api/guestbook/messages`, then adapt the friends page to consume nested replies while preserving its application-only filtering and login-required submission behavior.

**Tech Stack:** Go, SQLite, React, React Router, Tailwind utility classes, Node `node:test`, npm, git.

---

### Task 1: Add failing backend tests for threaded guestbook behavior

**Files:**
- Create: `D:\taozhiyy-monorepo\acg-api\guestbook_messages_test.go`
- Modify: `D:\taozhiyy-monorepo\acg-api\guestbook_messages.go` (later tasks only)
- Test: `D:\taozhiyy-monorepo\acg-api\guestbook_messages_test.go`

- [ ] **Step 1: Write the failing test file for top-level messages and replies**

```go
package main

import (
	"bytes"
	"database/sql"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	_ "modernc.org/sqlite"
)

func openGuestbookMessageTestDB(t *testing.T) *sql.DB {
	t.Helper()
	testDB, err := sql.Open("sqlite", ":memory:")
	if err != nil {
		t.Fatal(err)
	}
	if err := migrateAll(testDB); err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() { _ = testDB.Close() })
	return testDB
}

func withGuestbookTestDB(t *testing.T, fn func()) {
	t.Helper()
	prev := db
	db = openGuestbookMessageTestDB(t)
	t.Cleanup(func() { db = prev })
	fn()
}

func decodeJSONMap(t *testing.T, rr *httptest.ResponseRecorder) map[string]any {
	t.Helper()
	var out map[string]any
	if err := json.Unmarshal(rr.Body.Bytes(), &out); err != nil {
		t.Fatalf("decode response: %v", err)
	}
	return out
}

func TestGuestbookCreateTopLevelMessage(t *testing.T) {
	withGuestbookTestDB(t, func() {
		body := bytes.NewBufferString(`{"nickname":"Tao","content":"站点名称：A\n站点链接：https://a.test"}`)
		req := httptest.NewRequest(http.MethodPost, "/api/guestbook/messages", body)
		req.Header.Set("Content-Type", "application/json")
		req.RemoteAddr = "127.0.0.1:3456"
		rr := httptest.NewRecorder()

		guestbookCreateHandler(rr, req)

		if rr.Code != http.StatusOK {
			t.Fatalf("expected 200, got %d body=%s", rr.Code, rr.Body.String())
		}
		payload := decodeJSONMap(t, rr)
		item := payload["item"].(map[string]any)
		if item["parentId"].(float64) != 0 {
			t.Fatalf("expected top-level parentId 0, got %#v", item["parentId"])
		}
	})
}

func TestGuestbookCreateReplyWithParentID(t *testing.T) {
	withGuestbookTestDB(t, func() {
		first := bytes.NewBufferString(`{"nickname":"Tao","content":"站点名称：A\n站点链接：https://a.test"}`)
		req1 := httptest.NewRequest(http.MethodPost, "/api/guestbook/messages", first)
		req1.Header.Set("Content-Type", "application/json")
		req1.RemoteAddr = "127.0.0.1:3456"
		rr1 := httptest.NewRecorder()
		guestbookCreateHandler(rr1, req1)
		parentID := int(decodeJSONMap(t, rr1)["item"].(map[string]any)["id"].(float64))

		replyBody := bytes.NewBufferString(`{"nickname":"Reply","content":"已看到，晚点回链。","parentId":1}`)
		req2 := httptest.NewRequest(http.MethodPost, "/api/guestbook/messages", replyBody)
		req2.Header.Set("Content-Type", "application/json")
		req2.RemoteAddr = "127.0.0.1:4567"
		rr2 := httptest.NewRecorder()

		guestbookCreateHandler(rr2, req2)

		if rr2.Code != http.StatusOK {
			t.Fatalf("expected 200, got %d body=%s", rr2.Code, rr2.Body.String())
		}
		item := decodeJSONMap(t, rr2)["item"].(map[string]any)
		if int(item["parentId"].(float64)) != parentID {
			t.Fatalf("expected parentId %d, got %#v", parentID, item["parentId"])
		}
	})
}

func TestGuestbookRejectsReplyToMissingParent(t *testing.T) {
	withGuestbookTestDB(t, func() {
		body := bytes.NewBufferString(`{"nickname":"Reply","content":"找不到楼层","parentId":999}`)
		req := httptest.NewRequest(http.MethodPost, "/api/guestbook/messages", body)
		req.Header.Set("Content-Type", "application/json")
		req.RemoteAddr = "127.0.0.1:5678"
		rr := httptest.NewRecorder()

		guestbookCreateHandler(rr, req)

		if rr.Code != http.StatusBadRequest {
			t.Fatalf("expected 400, got %d body=%s", rr.Code, rr.Body.String())
		}
	})
}

func TestGuestbookListReturnsNestedReplies(t *testing.T) {
	withGuestbookTestDB(t, func() {
		requests := []string{
			`{"nickname":"Tao","content":"站点名称：A\n站点链接：https://a.test"}`,
			`{"nickname":"Friend","content":"已添加，来回访啦。","parentId":1}`,
		}
		for _, body := range requests {
			req := httptest.NewRequest(http.MethodPost, "/api/guestbook/messages", bytes.NewBufferString(body))
			req.Header.Set("Content-Type", "application/json")
			req.RemoteAddr = "127.0.0.1:3456"
			rr := httptest.NewRecorder()
			guestbookCreateHandler(rr, req)
			if rr.Code != http.StatusOK {
				t.Fatalf("seed create failed: %d %s", rr.Code, rr.Body.String())
			}
		}

		req := httptest.NewRequest(http.MethodGet, "/api/guestbook/messages?page=1&pageSize=20", nil)
		rr := httptest.NewRecorder()
		guestbookListHandler(rr, req)

		if rr.Code != http.StatusOK {
			t.Fatalf("expected 200, got %d body=%s", rr.Code, rr.Body.String())
		}
		payload := decodeJSONMap(t, rr)
		items := payload["items"].([]any)
		if len(items) != 1 {
			t.Fatalf("expected 1 top-level item, got %d", len(items))
		}
		top := items[0].(map[string]any)
		if int(top["replyCount"].(float64)) != 1 {
			t.Fatalf("expected replyCount 1, got %#v", top["replyCount"])
		}
		replies := top["replies"].([]any)
		if len(replies) != 1 {
			t.Fatalf("expected 1 reply, got %d", len(replies))
		}
	})
}
```

- [ ] **Step 2: Run the guestbook tests to verify they fail for the expected reason**

Run: `go test ./...`

Workdir: `D:\taozhiyy-monorepo\acg-api`

Expected: FAIL with missing `parentId` / `replyCount` fields or flat-list behavior, proving the new tests are covering unimplemented reply behavior.

- [ ] **Step 3: Commit the failing test checkpoint**

```bash
git -C D:\taozhiyy-monorepo add acg-api/guestbook_messages_test.go
git -C D:\taozhiyy-monorepo commit -m "test: cover guestbook threaded replies"
```

### Task 2: Implement backend schema and API support for replies

**Files:**
- Modify: `D:\taozhiyy-monorepo\acg-api\store.go`
- Modify: `D:\taozhiyy-monorepo\acg-api\guestbook_messages.go`
- Test: `D:\taozhiyy-monorepo\acg-api\guestbook_messages_test.go`

- [ ] **Step 1: Extend the schema with `parent_id` and an index**

```go
`CREATE TABLE IF NOT EXISTS guestbook_messages (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	user_id INTEGER,
	nickname TEXT NOT NULL,
	avatar TEXT NOT NULL DEFAULT '',
	content TEXT NOT NULL,
	content_hash TEXT NOT NULL DEFAULT '',
	ip_hash TEXT NOT NULL,
	ip_region TEXT NOT NULL DEFAULT '',
	ip_masked TEXT NOT NULL DEFAULT '',
	user_agent_hash TEXT NOT NULL DEFAULT '',
	parent_id INTEGER NOT NULL DEFAULT 0,
	status TEXT NOT NULL DEFAULT 'visible',
	is_login_user INTEGER NOT NULL DEFAULT 0,
	is_admin_user INTEGER NOT NULL DEFAULT 0,
	created_at TEXT NOT NULL,
	updated_at TEXT NOT NULL
);`,
`ALTER TABLE guestbook_messages ADD COLUMN parent_id INTEGER NOT NULL DEFAULT 0;`,
`CREATE INDEX IF NOT EXISTS idx_guestbook_messages_parent_created ON guestbook_messages(parent_id, created_at ASC);`,
```

- [ ] **Step 2: Add reply-aware response fields and request parsing**

```go
type guestbookMessageRow struct {
	ID            int64                `json:"id"`
	ParentID      int64                `json:"parentId"`
	Nickname      string               `json:"nickname"`
	Avatar        string               `json:"avatar"`
	Content       string               `json:"content"`
	IPRegion      string               `json:"ipRegion"`
	CreatedAt     string               `json:"createdAt"`
	IsLoginUser   bool                 `json:"isLoginUser"`
	IsAdminUser   bool                 `json:"isAdminUser,omitempty"`
	Status        string               `json:"status,omitempty"`
	IPMasked      string               `json:"ipMasked,omitempty"`
	UserAgentHash string               `json:"userAgentHash,omitempty"`
	ReplyCount    int                  `json:"replyCount,omitempty"`
	Replies       []guestbookMessageRow `json:"replies,omitempty"`
}

var body struct {
	Nickname string `json:"nickname"`
	Content  string `json:"content"`
	ParentID int64  `json:"parentId"`
}
```

- [ ] **Step 3: Validate `parentId`, persist replies, and return nested top-level items**

```go
func guestbookParentExists(parentID int64, admin bool) bool {
	if parentID <= 0 {
		return true
	}
	var count int
	query := `SELECT COUNT(*) FROM guestbook_messages WHERE id = ? AND status = 'visible'`
	if admin {
		query = `SELECT COUNT(*) FROM guestbook_messages WHERE id = ? AND status IN ('visible','hidden')`
	}
	_ = db.QueryRow(query, parentID).Scan(&count)
	return count > 0
}
```

```go
if body.ParentID < 0 {
	writeGuestbookErr(w, http.StatusBadRequest, "INVALID_PARENT", "回复楼层不正确")
	return
}
if body.ParentID > 0 && !guestbookParentExists(body.ParentID, isAdminUser(cu)) {
	writeGuestbookErr(w, http.StatusBadRequest, "INVALID_PARENT", "要回复的留言不存在或不可见")
	return
}
```

```go
res, err := db.Exec(
	`INSERT INTO guestbook_messages
	 (user_id, nickname, avatar, content, content_hash, ip_hash, ip_region, ip_masked, user_agent_hash, parent_id, status, is_login_user, is_admin_user, created_at, updated_at)
	 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'visible', ?, ?, ?, ?)`,
	userID, nickname, avatar, content, contentHash, ipHash, ipRegion, ipMasked, uaHash,
	body.ParentID, isLogin, isAdminUser, now, now,
)
```

```go
rows, err := db.Query(listQ+` ORDER BY parent_id ASC, id ASC`)
// Build top-level map, append rows with ParentID > 0 into their parent.Replies,
// then emit only ParentID == 0 items with ReplyCount filled.
```

- [ ] **Step 4: Re-run the guestbook tests to verify they pass**

Run: `go test ./...`

Workdir: `D:\taozhiyy-monorepo\acg-api`

Expected: PASS, including the new threaded reply tests and existing wallpaper/auth tests.

- [ ] **Step 5: Commit the backend reply implementation**

```bash
git -C D:\taozhiyy-monorepo add acg-api/store.go acg-api/guestbook_messages.go acg-api/guestbook_messages_test.go
git -C D:\taozhiyy-monorepo commit -m "feat: add threaded guestbook replies"
```

### Task 3: Add frontend helpers and tests for threading + friends-only filtering

**Files:**
- Create: `D:\taozhiyy-monorepo\main\src\components\friendsApplicationThreads.js`
- Create: `D:\taozhiyy-monorepo\main\src\components\friendsApplicationThreads.test.js`
- Modify: `D:\taozhiyy-monorepo\main\src\services\guestbookMessagesApi.js`

- [ ] **Step 1: Create a small utility that filters friend applications and preserves replies**

```js
export const FRIEND_APPLICATION_TEMPLATE_FIELDS = [
  "站点名称",
  "站点链接",
  "站点描述",
  "头像链接",
];

export function isFriendApplicationContent(content = "") {
  return FRIEND_APPLICATION_TEMPLATE_FIELDS.every((field) => content.includes(field));
}

export function normalizeFriendsThreads(items = []) {
  return items
    .filter((item) => item.parentId === 0 && isFriendApplicationContent(item.content))
    .map((item) => ({
      ...item,
      replies: Array.isArray(item.replies) ? item.replies : [],
      replyCount: typeof item.replyCount === "number"
        ? item.replyCount
        : Array.isArray(item.replies)
          ? item.replies.length
          : 0,
    }));
}
```

- [ ] **Step 2: Write failing Node tests for the helper behavior**

```js
import test from "node:test";
import assert from "node:assert/strict";

import {
  isFriendApplicationContent,
  normalizeFriendsThreads,
} from "./friendsApplicationThreads.js";

test("detects valid friend application content", () => {
  assert.equal(
    isFriendApplicationContent(
      "站点名称：桃之夭夭\n站点链接：https://taozhiyy.top\n站点描述：桃之夭夭的小屋\n头像链接：https://img.test/1.png"
    ),
    true
  );
});

test("keeps replies under valid friend applications", () => {
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

  assert.equal(threads.length, 1);
  assert.equal(threads[0].replies.length, 1);
  assert.equal(threads[0].replyCount, 1);
});
```

- [ ] **Step 3: Extend the service layer without breaking callers**

```js
export async function postGuestbookMessage(body) {
  const res = await fetch("/api/guestbook/messages", {
    method: "POST",
    credentials: "include",
    headers: JSON_HEADERS,
    body: JSON.stringify(body),
  });
  return parseJson(res);
}

export function postGuestbookReply(parentId, content) {
  return postGuestbookMessage({ parentId, content });
}
```

- [ ] **Step 4: Run the frontend helper tests to verify they pass**

Run: `node --test src/components/friendsApplicationThreads.test.js src/components/heroImageState.test.js`

Workdir: `D:\taozhiyy-monorepo\main`

Expected: PASS for the new helper tests and the existing hero image test.

- [ ] **Step 5: Commit the frontend threading helpers**

```bash
git -C D:\taozhiyy-monorepo add main/src/components/friendsApplicationThreads.js main/src/components/friendsApplicationThreads.test.js main/src/services/guestbookMessagesApi.js
git -C D:\taozhiyy-monorepo commit -m "test: cover friends application thread helpers"
```

### Task 4: Rebuild `FriendsApplicationBoard.jsx` into a real comment thread UI

**Files:**
- Modify: `D:\taozhiyy-monorepo\main\src\components\FriendsApplicationBoard.jsx`
- Modify: `D:\taozhiyy-monorepo\main\src\services\authApi.js` (only if a small auth helper is needed)
- Use: `D:\taozhiyy-monorepo\main\src\components\friendsApplicationThreads.js`

- [ ] **Step 1: Refactor state shape to support replies and active reply editors**

```jsx
const FRIEND_TEMPLATE = `站点名称：
站点链接：
站点描述：
头像链接：
已添加本站友链：是`;

const [entries, setEntries] = useState([]);
const [content, setContent] = useState(FRIEND_TEMPLATE);
const [replyDrafts, setReplyDrafts] = useState({});
const [replyTargetId, setReplyTargetId] = useState(null);
const [expandedThreads, setExpandedThreads] = useState({});
const [replySubmittingId, setReplySubmittingId] = useState(null);
```

- [ ] **Step 2: Load nested threads through the helper and keep login gating**

```jsx
const loadEntries = async () => {
  setLoading(true);
  const { ok, data } = await fetchGuestbookMessages(1, 30);
  if (!ok) {
    setEntries([]);
    setLoading(false);
    return;
  }
  setEntries(normalizeFriendsThreads(asList(data.items ?? data)));
  setLoading(false);
};
```

```jsx
if (!user) {
  setError("请先登录后再提交友链申请。");
  return;
}
```

- [ ] **Step 3: Add reply submit flow and optimistic thread update**

```jsx
const onReplySubmit = async (parentId) => {
  if (!user) {
    setError("请先登录后再回复这条友链申请。");
    return;
  }
  const nextContent = (replyDrafts[parentId] || "").trim();
  if (!nextContent) return;

  setReplySubmittingId(parentId);
  const { ok, data } = await postGuestbookReply(parentId, nextContent);
  setReplySubmittingId(null);

  if (!ok) {
    setError(data.message || "回复没有提交成功，请稍后再试。");
    return;
  }

  setEntries((prev) =>
    prev.map((item) =>
      item.id === parentId
        ? {
            ...item,
            replies: [...(item.replies || []), data.item],
            replyCount: (item.replyCount || 0) + 1,
          }
        : item
    )
  );
  setExpandedThreads((prev) => ({ ...prev, [parentId]: true }));
  setReplyDrafts((prev) => ({ ...prev, [parentId]: "" }));
  setReplyTargetId(null);
};
```

- [ ] **Step 4: Render thread cards, expand/collapse controls, and nested reply blocks**

```jsx
{entries.map((item) => {
  const isExpanded = !!expandedThreads[item.id];
  const replies = item.replies || [];
  return (
    <article key={item.id} className="rounded-[24px] border border-[#E7D8C7] bg-white/90 p-5 shadow-[0_16px_42px_rgba(95,75,82,0.08)]">
      <div className="flex items-start justify-between gap-4">
        <div>
          <p className="text-base font-semibold text-[#2B2B2B]">{item.nickname}</p>
          <p className="mt-1 text-xs text-[#8A7C74]">{formatTime(item.createdAt)}</p>
        </div>
        <button
          type="button"
          onClick={() => setReplyTargetId((current) => (current === item.id ? null : item.id))}
          className="rounded-full border border-[#E7D8C7] px-3 py-1 text-xs font-semibold text-[#7B5C61]"
        >
          回复
        </button>
      </div>

      <p className="mt-4 whitespace-pre-wrap text-[15px] leading-8 text-[#3F3A37]">{item.content}</p>

      {item.replyCount > 0 && (
        <button
          type="button"
          onClick={() => setExpandedThreads((prev) => ({ ...prev, [item.id]: !isExpanded }))}
          className="mt-4 text-sm font-medium text-[#B76E79]"
        >
          {isExpanded ? "收起回复" : `展开回复（${item.replyCount}）`}
        </button>
      )}

      {isExpanded && replies.length > 0 && (
        <div className="mt-4 space-y-3 border-l border-[#F0E3D8] pl-4">
          {replies.map((reply) => (
            <div key={reply.id} className="rounded-[18px] bg-[#FFF8F1] p-4">
              <p className="text-sm font-semibold text-[#5F4B52]">{reply.nickname}</p>
              <p className="mt-2 whitespace-pre-wrap text-sm leading-7 text-[#5C5652]">{reply.content}</p>
            </div>
          ))}
        </div>
      )}
    </article>
  );
})}
```

- [ ] **Step 5: Run a production build for `main`**

Run: `npm run build`

Workdir: `D:\taozhiyy-monorepo\main`

Expected: PASS, with no JSX syntax, unused import, or state update errors.

- [ ] **Step 6: Commit the threaded board UI**

```bash
git -C D:\taozhiyy-monorepo add main/src/components/FriendsApplicationBoard.jsx main/src/services/authApi.js
git -C D:\taozhiyy-monorepo commit -m "feat: add threaded friends application board"
```

### Task 5: Refresh `FriendsPage.jsx` and `index.css` to match the new poetic direction

**Files:**
- Modify: `D:\taozhiyy-monorepo\main\src\pages\FriendsPage.jsx`
- Modify: `D:\taozhiyy-monorepo\main\src\index.css`

- [ ] **Step 1: Replace placeholder friend cards with real featured card data**

```jsx
const friendCards = [
  {
    name: "XingHuiSamaの宝藏之地",
    desc: "今天我也要学习吗",
    url: "https://www.xinghuisama.top",
    avatar: "https://bu.dusays.com/2026/03/24/69c1e38ac1846.jpg",
    note: "FIRST FRIEND",
  },
];
```

- [ ] **Step 2: Rewrite the hero and section copy into the more poetic site-native tone**

```jsx
<header className="mx-auto max-w-3xl text-center">
  <p className="text-[11px] font-semibold uppercase tracking-[0.34em] text-[#B76E79]">
    Friends Page
  </p>
  <h1 className="mt-4 text-4xl font-semibold tracking-[-0.04em] text-[#2B2B2B] md:text-6xl">
    友链
  </h1>
  <p className="mt-5 text-base leading-8 text-[#6B7280] md:text-lg">
    风会替信纸赶路，链接会替心意停留。若你也愿意把小屋的灯留给远方的人，这里便是交换名字的地方。
  </p>
</header>
```

- [ ] **Step 3: Make the friend card and own-link block feel warmer and more editorial**

```jsx
<a
  href={friend.url}
  target="_blank"
  rel="noreferrer"
  className="group block rounded-[28px] border border-[#F0E3D8] bg-[linear-gradient(180deg,rgba(255,255,255,0.96),rgba(255,248,241,0.94))] p-5 shadow-[0_18px_44px_rgba(95,75,82,0.10)] transition duration-300 hover:-translate-y-1"
>
  <div className="flex items-center gap-4">
    <img src={friend.avatar} alt={friend.name} className="h-16 w-16 rounded-[20px] object-cover" />
    <div>
      <p className="text-lg font-semibold text-[#2B2B2B]">{friend.name}</p>
      <p className="mt-1 text-sm text-[#6B7280]">{friend.desc}</p>
    </div>
  </div>
</a>
```

```jsx
<div className="friends-verse-panel mt-6 overflow-hidden rounded-[28px] border border-[#F2E6C9] bg-white/92">
  <div className="friends-verse-panel__chrome">
    <span className="h-3 w-3 rounded-full bg-[#FF8FAB]" />
    <span className="h-3 w-3 rounded-full bg-[#FFD43B]" />
    <span className="h-3 w-3 rounded-full bg-[#74C0FC]" />
  </div>
  <pre className="overflow-x-auto px-5 py-5 text-sm leading-8 text-[#2B2B2B]">{copyBlock}</pre>
</div>
```

- [ ] **Step 4: Add a small supporting CSS surface only where utility classes get noisy**

```css
.friends-verse-panel {
  box-shadow:
    0 18px 48px rgba(95, 75, 82, 0.12),
    inset 0 1px 0 rgba(255, 255, 255, 0.7);
  background:
    radial-gradient(circle at top right, rgba(255, 224, 205, 0.32), transparent 30%),
    linear-gradient(180deg, rgba(255, 253, 248, 0.95), rgba(255, 247, 240, 0.94));
}

.friends-verse-panel__chrome {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  border-bottom: 1px solid rgba(242, 230, 201, 0.9);
  background: linear-gradient(180deg, rgba(255, 253, 248, 0.98), rgba(255, 246, 238, 0.92));
  padding: 0.9rem 1rem;
}
```

- [ ] **Step 5: Re-run the frontend helper tests and production build**

Run: `node --test src/components/friendsApplicationThreads.test.js src/components/heroImageState.test.js`

Expected: PASS

Run: `npm run build`

Workdir: `D:\taozhiyy-monorepo\main`

Expected: PASS

- [ ] **Step 6: Commit the page visual refresh**

```bash
git -C D:\taozhiyy-monorepo add main/src/pages/FriendsPage.jsx main/src/index.css
git -C D:\taozhiyy-monorepo commit -m "feat: refresh friends page atmosphere"
```

### Task 6: Final verification and handoff

**Files:**
- Modify: `D:\taozhiyy-monorepo\docs\superpowers\specs\2026-06-05-friends-hub-design.md` (only if the implementation intentionally diverges)
- Modify: `D:\taozhiyy-monorepo\docs\superpowers\plans\2026-06-05-friends-hub-implementation.md` (optional progress updates)

- [ ] **Step 1: Check repo status before final wrap-up**

Run: `git status --short`

Workdir: `D:\taozhiyy-monorepo`

Expected: only intended files for threaded friends-page work are modified or staged.

- [ ] **Step 2: Re-run all relevant verification commands**

Run: `go test ./...`

Workdir: `D:\taozhiyy-monorepo\acg-api`

Expected: PASS

Run: `node --test src/components/friendsApplicationThreads.test.js src/components/heroImageState.test.js`

Workdir: `D:\taozhiyy-monorepo\main`

Expected: PASS

Run: `npm run build`

Workdir: `D:\taozhiyy-monorepo\main`

Expected: PASS

- [ ] **Step 3: Spot-check for threaded reply references**

Run: `rg -n "parentId|replyCount|replies|replyDrafts|XingHuiSama" acg-api main/src`

Workdir: `D:\taozhiyy-monorepo`

Expected: results show the new backend fields, frontend helper usage, and updated featured friend card content.

- [ ] **Step 4: Prepare the final code summary**

```bash
git -C D:\taozhiyy-monorepo diff --stat
```

- [ ] **Step 5: Commit the final integrated state if everything is green**

```bash
git -C D:\taozhiyy-monorepo add acg-api main docs/superpowers/specs/2026-06-05-friends-hub-design.md docs/superpowers/plans/2026-06-05-friends-hub-implementation.md
git -C D:\taozhiyy-monorepo commit -m "feat: add threaded friends applications"
```
