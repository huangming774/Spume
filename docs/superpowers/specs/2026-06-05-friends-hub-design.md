# Friends Page + Threaded Application Board Design

## Goal

Refresh `/friends` into a quieter, more poetic page that keeps only the three sections the user wants, while upgrading the reused guestbook message system into a real threaded reply model for friend-link applications.

This iteration replaces the earlier friends-hub scope. The active scope is now:

1. `小伙伴卡片`
2. `我的友链`
3. `申请友链的留言区`

## Product Scope

### In Scope

- Keep `/friends` as a three-section page only.
- Rewrite the top copy and section framing so it feels like `桃之夭夭`'s own in-site voice, with more poetic wording and less product-description tone.
- Replace placeholder friend cards with a real first featured friend card:
  - Name: `XingHuiSamaの宝藏之地`
  - Desc: `今天我也要学习吗`
  - URL: `https://www.xinghuisama.top`
  - Avatar: `https://bu.dusays.com/2026/03/24/69c1e38ac1846.jpg`
- Preserve `桃之夭夭`'s own friend-link information in the middle section.
- Keep the application board on the existing message system path, but require login before posting friend-link applications.
- Upgrade the message system to support real replies under a parent message.
- Make the application board feel like a free-form comment area with expandable replies, not a plain form list.

### Out of Scope

- No new separate friend-link moderation panel.
- No multi-level deep nesting beyond one visible reply level in the UI for this pass.
- No changes to unrelated pages outside the shared guestbook message capability.
- No migration of old friend-link data from blog or build in this pass.

## User Experience

### 1. Page Tone

The page should feel soft, airy, and literary rather than technical. The current structural simplicity is correct, but the copy should shift from explanatory UI language into short poetic phrases and warm guidance.

The hero area should:

- Keep the page anchored as `/friends`
- Use a quieter, poetic title and supporting line
- Introduce the page as a place where links are exchanged slowly and intentionally

### 2. 小伙伴卡片

This section becomes a curated friends area rather than a placeholder board.

Requirements:

- The first visible card must be `XingHuiSamaの宝藏之地`
- The card should include avatar, name, short intro, and clickable site link
- The layout should feel like a warm scrapbook / stationery card that still matches the site
- Additional placeholder cards may remain only if they still feel intentional and not empty-demo-like

### 3. 我的友链

This section stays lightweight and copy-friendly.

Content remains the same canonical four-line friend-link info for `桃之夭夭`.

Presentation requirements:

- Preserve the four-line structure
- Keep it easy to copy
- Visually soften it so it feels closer to a calm note card or polished code snippet window, not a harsh dev block

Canonical content:

```yaml
name: 桃之夭夭
desc: 桃之夭夭的小屋
url: https://taozhiyy.top
avatar: https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/1.png
```

### 4. 申请友链留言区

This area should look like a real comment wall.

Requirements:

- Eyebrow / label direction can use `Comment Style`
- Title direction can use `留言申请`
- Intro copy must clearly say the section reuses the guestbook interface, but applications require login
- Logged-in users see the application template prefilled
- Logged-out users see a login prompt and disabled submission
- Empty state remains:
  - `还没有友链申请留言，欢迎留下第一条。`

The default application content should follow this structure:

```text
站点名称：
站点链接：
站点描述：
头像链接：
已添加本站友链：是
```

When the current logged-in identity is `桃之夭夭`, the UI should explicitly surface the identity framing:

- `以 桃之夭夭 身份提交友链申请`

## Reply Model

### Recommendation

Use real persisted replies, not fake `@someone` formatting.

### Backend Behavior

Each guestbook message may optionally point to a parent message.

Rules:

- Top-level friend-link applications have no parent
- Replies must point to an existing visible-or-manageable parent message
- Reply creation also requires login for the friends page flow
- For this iteration, replies can themselves be stored as messages, but the UI will present one visible nested level under top-level application items

### Frontend Behavior

Each top-level application card should support:

- `回复` action
- Expand / collapse replies when replies exist
- Inline reply editor beneath the parent message
- Reply submission state and error feedback

Displayed structure:

- Top-level application card
- Its replies listed beneath it with lighter visual weight
- Reply input shown only for the active target

This should feel like a drop-down conversational stack rather than a flat list.

## Data Design

### Current Limitation

The existing `guestbook_messages` table and API support only flat messages. There is no current `parent_id` or reply relation.

### Required Schema Extension

Extend `guestbook_messages` with reply metadata.

Minimum addition:

- `parent_id INTEGER NOT NULL DEFAULT 0`

Interpretation:

- `0` means top-level message
- Positive value means reply to that message

Recommended DB support:

- Index on `(parent_id, created_at)`

This is enough for a first real threaded model without introducing a separate replies table.

## API Design

### Existing Endpoints to Reuse

- `GET /api/guestbook/messages`
- `POST /api/guestbook/messages`

### GET Response Shape

The list endpoint should return enough information for the friends page to build a threaded view.

Recommended response strategy:

- Return only top-level items in the main list
- Attach their replies as a nested `replies` array
- Include `replyCount` for UI display

This keeps the frontend simpler than flattening and re-grouping everything manually.

### POST Request Shape

Keep the current payload compatible and extend it.

Top-level application:

```json
{ "content": "..." }
```

Reply:

```json
{ "content": "...", "parentId": 123 }
```

Validation:

- `parentId` must be absent or a positive integer
- If `parentId` is provided, the parent message must exist and not be deleted
- Friends-page posting should still enforce login on the frontend
- Backend should remain robust if other clients call the endpoint directly

## UI Composition

### FriendsPage.jsx

Responsibilities:

- Hero copy refresh
- Render the three required sections only
- Inject updated friend card data
- Keep the own-link section centered and copy-friendly
- Host the threaded application board beneath the other sections

### FriendsApplicationBoard.jsx

Responsibilities:

- Fetch threaded application entries
- Filter or frame content so the board remains friend-link oriented
- Gate top-level submission behind login
- Render top-level items and nested replies
- Handle reply target state
- Submit top-level applications and replies through the same API

### index.css

Responsibilities:

- Add or refine supporting visual tokens for the poetic friends page
- Add comment-wall details if utility classes alone become noisy
- Keep the style consistent with the main site instead of introducing a foreign system

## Content Filtering

The current board filters visible messages to application-like content. That behavior should remain, but it must also account for replies.

Recommended logic:

- Top-level items shown on the friends page must match the application template pattern
- Replies should be shown if their parent is an included top-level application
- Replies themselves do not need to match the application template

This prevents ordinary guestbook chatter from polluting the board while still allowing real conversation under valid applications.

## Error Handling

### Submission Errors

- Keep current rate-limit and generic error handling
- Add a clear invalid-parent failure path for replies
- If login expires mid-session, prompt the user to log in again and keep the draft content if practical

### Loading States

- Keep the current loading state, but make it read like comment loading rather than raw data fetch
- Reply areas should have local submitting indicators so one reply does not block the entire board more than necessary

## Testing Plan

### Backend

Add tests before implementation for:

- Creating a top-level message
- Creating a reply with valid `parentId`
- Rejecting reply creation when parent does not exist
- Listing top-level messages with nested replies
- Preserving current flat-message behavior for non-reply callers

### Frontend

At minimum verify:

- `/friends` renders the new three-section layout
- The new featured friend card appears correctly
- Logged-out users cannot submit applications or replies
- Logged-in users can open a reply editor under a message
- Reply submission updates the visible thread correctly

### Build Verification

- `main`: `npm run build`
- `acg-api`: relevant Go test command for guestbook reply logic

## Risks and Mitigations

### Risk: Existing guestbook consumers expect a flat list

Mitigation:

- Keep response compatibility carefully, or confine nested reply rendering to additive fields
- Avoid removing existing top-level fields used elsewhere

### Risk: Thread rendering gets noisy on mobile

Mitigation:

- Limit visible nesting depth
- Use lighter reply cards, tighter spacing, and collapsible reply groups

### Risk: Friends-page login rule differs from general guestbook behavior

Mitigation:

- Enforce login in the friends-page UI
- Keep backend validation generic and safe
- Do not silently allow unauthenticated submission from this page

## Acceptance Criteria

The work is done when:

- `/friends` contains only `小伙伴卡片`、`我的友链`、`申请友链的留言区`
- The page copy and visual framing feel more poetic and site-native
- `XingHuiSamaの宝藏之地` is the first real friend card shown
- `我的友链` presents `桃之夭夭`'s four-line info in a copy-friendly card
- The application board requires login for submission
- Users can reply beneath an existing friend-link application
- Replies persist after refresh
- The board still excludes unrelated non-application top-level guestbook messages
