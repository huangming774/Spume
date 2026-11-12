# PWA Owner Login Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Require the existing AI owner login before showing the installable app on `app.taozhiyy.top`.

**Architecture:** Add a small host/session decision helper under `main/src/pwa/`, then wrap the React routes with a gate that only activates on the PWA host. The gate reuses `/api/auth/me` and the existing AI auth modal opened by the `blog-ai-open` event.

**Tech Stack:** React, Vite, Node test runner, existing acg-api auth endpoints.

---

### Task 1: Gate Decision Helper

**Files:**
- Create: `main/src/pwa/appAccessGate.js`
- Create: `main/src/pwa/appAccessGate.test.js`

- [x] Write tests for host gating and owner-only access.
- [x] Implement helper functions.

### Task 2: React Gate

**Files:**
- Create: `main/src/pwa/AppAccessGate.jsx`
- Modify: `main/src/App.jsx`
- Modify: `main/src/pages/LoginPage.jsx`

- [x] Wrap routes in the gate.
- [x] Open the existing AI login modal for unauthenticated PWA users.
- [x] Keep the public root domain unchanged.

### Task 3: Verification And Deploy

- [ ] Run focused Node tests.
- [ ] Build `main`.
- [ ] Upload build output to the server.
- [ ] Verify `app.taozhiyy.top` requires owner login and `taozhiyy.top` remains public.
