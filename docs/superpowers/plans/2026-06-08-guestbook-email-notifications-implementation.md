# Guestbook Email Notifications Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add private email collection for friends-page messages and best-effort SMTP notifications for new messages and owner replies.

**Architecture:** Extend the existing `guestbook_messages` model with a private `contact_email` field and keep it out of public list responses. Add a small Go mail notifier with dependency injection for tests, then call it after successful message creation. Update the friends application board to collect nickname/email only where needed while leaving the generic guestbook wall unchanged.

**Tech Stack:** Go 1.22 `net/http` + SQLite, React/Vite, GitHub Actions deployment, QQ-compatible SMTP via Go standard library.

---

### Task 1: Backend Data And Validation

**Files:**
- Modify: `acg-api/store.go`
- Modify: `deploy/guestbook.sql`
- Modify: `acg-api/guestbook_user.go`
- Modify: `acg-api/guestbook_messages.go`
- Test: `acg-api/guestbook_messages_test.go`
- Test: `acg-api/store_test.go`

- [ ] Add failing Go tests for `contact_email` migration, friends anonymous email requirement, friends logged-in override/fallback, and guestbook no-email compatibility.
- [ ] Run `go test ./...` from `acg-api`; expect the new tests to fail because the column and validation do not exist yet.
- [ ] Add `contact_email TEXT NOT NULL DEFAULT ''` to table creation, migrations, and deployment SQL.
- [ ] Add `Email string` to `currentUser` so existing site auth can provide fallback email.
- [ ] Parse `contactEmail` in `POST /api/guestbook/messages` and store it only for top-level friends messages according to the spec.
- [ ] Run `go test ./...` from `acg-api`; expect the backend tests to pass.

### Task 2: Backend Mail Notifier

**Files:**
- Create: `acg-api/mail_notifier.go`
- Test: `acg-api/mail_notifier_test.go`
- Modify: `acg-api/guestbook_messages.go`
- Modify: `acg-api/.env.example`
- Modify: `.github/workflows/deploy.yml`
- Modify: `deploy/sync-auth-env.sh`
- Test: `acg-api/guestbook_messages_test.go`
- Test: `tools/test_deploy_workflow.py`

- [ ] Add failing tests for disabled SMTP config, best-effort owner notification, owner reply recipient resolution from parent `contact_email`, fallback to parent user account email, and mail failure not breaking message creation.
- [ ] Add failing deployment workflow tests that assert SMTP secrets are synced into the server env fragment.
- [ ] Run `go test ./...` from `acg-api` and `python -m unittest tools.test_deploy_workflow`; expect the new tests to fail.
- [ ] Implement `mail_notifier.go` with a `siteMailer` interface, SMTP config loading, text email composition, and injectable sender for tests.
- [ ] Call notification hooks after successful guestbook insert.
- [ ] Add SMTP placeholder variables to `.env.example`.
- [ ] Add SMTP GitHub secrets to `.github/workflows/deploy.yml` env sync and `deploy/sync-auth-env.sh`.
- [ ] Run `go test ./...` from `acg-api` and `python -m unittest tools.test_deploy_workflow`; expect tests to pass.

### Task 3: Friends-Page Form UI

**Files:**
- Modify: `main/src/components/FriendsApplicationBoard.jsx`
- Modify: `main/src/services/guestbookMessagesApi.js` if needed

- [ ] Add local form state for `nickname` and `contactEmail`.
- [ ] For anonymous friends-page users, render nickname and required private email fields above the textarea, keep the textarea enabled, and require all three fields before submit.
- [ ] For logged-in users, render an optional private reply email field under the identity line; submit it only when filled.
- [ ] Keep reply behavior login-gated and keep the generic guestbook wall unchanged.
- [ ] Run `npm run build` from `main`; expect the build to pass.

### Task 4: Verification, Preview, And Deploy Gate

**Files:**
- No source edits unless verification finds a defect.

- [ ] Run backend verification: `go test ./...` in `acg-api`.
- [ ] Run frontend verification: `npm run build` in `main`.
- [ ] Run deployment workflow verification: `python -m unittest tools.test_deploy_workflow tools.test_remote_install_acg_api`.
- [ ] Start local preview with `npm run dev -- --host 127.0.0.1`.
- [ ] Open `/friends` in the in-app browser and inspect desktop/mobile layout for nickname/email placement.
- [ ] Pause before deployment and ask the user to approve the friends-page preview.
- [ ] After approval, configure deployment SMTP secrets outside the repository and trigger deployment.
