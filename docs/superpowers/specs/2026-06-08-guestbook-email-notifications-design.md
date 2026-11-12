# Guestbook Email Notifications Design

## Goal

Add email notifications for guestbook and friends-page messages without exposing visitor email addresses publicly or committing mail credentials.

The site will use a dedicated SMTP sender mailbox configured through environment variables. Any new message should notify the site owner. Replies should notify the original message author when the backend can determine a recipient email.

## Scope

- Notify the owner when a new top-level message or reply is created in either guestbook channel.
- Notify a message author when the owner/admin replies to their message.
- Add private email collection to the friends-page application board only.
- Reuse the existing site account login/session system used by the AI feature; do not add a separate friends-page login.
- Keep the generic guestbook wall unchanged: no required email field, no extra public email display.
- Store SMTP credentials only in deployment environment variables or local uncommitted `.env` files.

## Sender Configuration

The backend reads SMTP configuration from environment variables:

- `SMTP_HOST`
- `SMTP_PORT`
- `SMTP_USER`
- `SMTP_PASS`
- `SMTP_FROM_NAME`
- `MAIL_NOTIFY_TO`

For QQ Mail, the production values should use the QQ SMTP host, an SMTP authorization code, and the dedicated sender mailbox. The authorization code must not be committed to the repository. Because an authorization code was shared in chat during planning, it should be rotated before production deployment.

If SMTP configuration is incomplete, message creation should still succeed and the backend should skip sending mail with a logged warning.

## Data Model

Extend `guestbook_messages` with a private recipient email column:

- `contact_email TEXT NOT NULL DEFAULT ''`

This column is never returned in public list responses. It may be visible only through owner/admin APIs if future moderation tooling needs it. For this implementation, the guestbook list API should omit it.

The column stores the email chosen for future reply notification. "Logged-in" means the existing site account login used by the AI feature, not a separate friends-page identity system:

- Logged-in friends-page message with form email: use the form email.
- Logged-in friends-page message without form email: use the existing account email from the site auth system.
- Anonymous friends-page message: require the form email and store it.
- Generic guestbook message: do not require an email; store an empty value unless a future caller explicitly supports it.

## Friends-Page Form Rules

The friends-page board (`channel = "friends"`) gets contact fields for top-level messages:

- Anonymous user:
  - `nickname` is required.
  - `contactEmail` is required.
  - The UI explains that the email is private and used only for reply notifications.
- Logged-in user:
  - `contactEmail` is optional.
  - If provided, it overrides the existing site account email for reply notifications.
  - If omitted, the backend uses the existing site account email.
  - This is the same login state already used elsewhere on the site, including AI access; the friends page must not introduce its own login model.

The generic guestbook wall (`channel = "guestbook"`) should not show or require these fields.

Replies do not need a contact email field. A reply's notification recipient is resolved from the parent message.

## Backend Flow

### Creating A Message

`POST /api/guestbook/messages` accepts an optional `contactEmail` field.

Validation:

- Normalize and validate `contactEmail` with the existing email validator.
- If `channel = "friends"`, `parentId = 0`, and the user is anonymous, require a valid `contactEmail`.
- If `channel = "friends"`, `parentId = 0`, and the user is logged in through the existing site auth session, accept a valid optional `contactEmail`; otherwise fall back to the user's account email.
- If `channel = "guestbook"`, do not require `contactEmail`.
- For replies (`parentId > 0`), ignore submitted `contactEmail`.

After the database insert succeeds:

- Send owner notification for every created message or reply.
- If the message is an owner/admin reply, send a reply notification to the parent message's recipient email when available.

Mail sending should be best-effort. If mail sending fails, the API should still return success for the saved message and log the mail error.

### Reply Recipient Resolution

When an owner/admin reply is created:

1. Load the parent message by `parentId`.
2. If the parent has `contact_email`, send to that email.
3. Otherwise, if the parent has `user_id`, load that user's account email and send to it.
4. If no recipient is found, skip the reply notification.
5. Do not notify the owner about their own reply if that would duplicate the owner notification in an annoying way; owner notifications remain useful for audit, but reply-recipient mail must not send back to the sender address.

## Email Content

Owner notification subject examples:

- `New guestbook message`
- `New friends-page message`
- `New reply on friends-page`

Reply notification subject example:

- `Your message has a new reply`

Email bodies should include:

- Site name.
- Message channel.
- Author nickname.
- Message content.
- Reply content, when applicable.
- A link back to the relevant page.

Keep the first version text-only for reliability. HTML templates can be added later.

## Privacy

- Do not display `contactEmail` in public UI.
- Do not include visitor email in public API responses.
- Do not log full SMTP credentials.
- Avoid including visitor email in owner notification bodies unless there is a clear moderation need. The owner can receive message content and author nickname without exposing email in routine notifications.

## Testing

Backend tests:

- Friends anonymous top-level message rejects missing email.
- Friends anonymous top-level message accepts valid email and stores it.
- Friends logged-in top-level message uses submitted email when provided.
- Friends logged-in top-level message falls back to login account email when omitted.
- Generic guestbook message does not require email.
- Public list responses do not include `contactEmail`.
- Owner/admin reply resolves recipient from parent `contact_email`.
- Owner/admin reply falls back to parent user account email.
- Mail failures do not fail message creation.

Frontend tests:

- Friends board shows nickname and email fields for anonymous users.
- Friends board requires email for anonymous users before submitting.
- Friends board shows optional private email field for logged-in users.
- Generic guestbook wall remains unchanged.

## Deployment Notes

Required production environment variables:

- `SMTP_HOST`
- `SMTP_PORT`
- `SMTP_USER`
- `SMTP_PASS`
- `SMTP_FROM_NAME`
- `MAIL_NOTIFY_TO`

Before enabling production mail, generate a fresh SMTP authorization code and replace the planning-time code. Then deploy the environment variables on the API host and run one manual end-to-end test:

1. Anonymous friends-page message sends owner notification.
2. Owner/admin reply sends visitor notification.
3. Generic guestbook message still works without an email field.
