# COS Upload Design

Date: 2026-06-07

## Context

The current owner upload flow writes image files to `ACG_DATA_DIR/owner-uploads` and returns local backend URLs such as `/api/owner/uploads/...`. That is enough for local persistence, but it does not match the intended production workflow:

- `gallery` images should live in Tencent COS and be referenced by public COS URLs.
- article images should also live in Tencent COS, but under a separate path strategy from gallery assets.
- desktop usage should continue to support existing PicGo-hosted URLs without forcing a re-upload.
- mobile usage should work without PicGo by uploading through the backend.

The existing owner article publish flow already uses backend-only secrets and GitHub Actions secrets. The COS upload flow should follow the same model: secrets stay on the backend and never enter frontend code or git history.

## Goals

- Add a real backend Tencent COS upload path for owner-managed assets.
- Support both asset kinds:
  - `gallery`
  - `article`
- Keep gallery and article assets separated in COS paths.
- Support direct PicGo URL reuse in the owner console without re-uploading.
- Return final public asset URLs that can be stored in drafts or gallery data.
- Keep the implementation aligned with current owner controller patterns and test style.

## Non-Goals

- No direct browser-to-COS upload from the frontend.
- No client-side COS credential exposure.
- No full gallery CRUD redesign in this change.
- No migration of existing hard-coded COS URLs already used by the site.
- No automatic importing of arbitrary remote images into COS when the user pastes a URL.

## Options Considered

### 1. Backend direct upload only

All desktop and mobile uploads go through the backend and are stored in COS.

Pros:

- One consistent path.
- Simplest data model.

Cons:

- Does not preserve the user's current PicGo workflow for desktop use.

### 2. Backend direct upload plus PicGo URL support

Users can either upload a file to the backend or paste an existing public image URL.

Pros:

- Matches current usage on both desktop and mobile.
- Keeps mobile unblocked.
- Avoids forcing a workflow change for desktop.

Cons:

- Slightly more validation logic in the frontend and backend.

### 3. Gallery-only COS support first

Only gallery uploads move to COS now; article images stay on the current local upload path.

Pros:

- Smaller change set.

Cons:

- Preserves a split asset model.
- Requires another follow-up change to unify behavior later.

## Recommendation

Use option 2.

This is the smallest design that meets the real usage pattern:

- mobile can upload files through the backend;
- desktop can either upload files or paste existing PicGo-hosted URLs;
- the backend remains the only place that knows COS credentials;
- gallery and article assets can share one uploader implementation while keeping separate path rules.

## COS Configuration

The backend will read these environment variables:

- `TENCENT_COS_SECRET_ID`
- `TENCENT_COS_SECRET_KEY`
- `TENCENT_COS_BUCKET`
- `TENCENT_COS_REGION`
- optional `TENCENT_COS_BASE_URL`

Behavior:

- if `TENCENT_COS_BASE_URL` is set, returned asset URLs use it;
- otherwise the returned public URL defaults to `https://<bucket>.cos.<region>.myqcloud.com`.

These values must live in server environment files and GitHub Actions secrets only. They must not be committed to the repo and must not be exposed to frontend code.

## Backend Design

### Shared uploader

Add a Tencent COS uploader service in `acg-api` with these responsibilities:

- load and validate COS config from environment;
- accept a file payload plus upload metadata;
- build the target object key;
- upload bytes to COS with the detected content type;
- return:
  - public URL
  - object key
  - mime type
  - file size

The uploader should be injectable enough for tests so the controller layer can be tested without real network calls.

### New owner asset endpoint

Add a new owner endpoint:

- `POST /api/owner/assets`

Request:

- multipart form
- `file`: required for direct upload
- `kind`: required, one of `gallery` or `article`
- `album`: required when `kind=gallery`, ignored otherwise

Response:

```json
{
  "ok": true,
  "item": {
    "kind": "gallery",
    "url": "https://...",
    "path": "gallery/misaka/20260607-123000-uuid.png",
    "mimeType": "image/png",
    "size": 12345
  }
}
```

Error cases:

- unauthenticated or non-owner session
- missing file
- invalid `kind`
- missing `album` for gallery uploads
- unsupported file type
- file too large
- COS not configured
- upstream COS upload failure

### Existing local upload endpoint

Current `/api/owner/uploads` behavior should be retired from the owner console flow and replaced by `/api/owner/assets`.

The local endpoint can either:

- stay temporarily for backward compatibility but unused by the current UI, or
- be removed if nothing else depends on it.

For this change, the safer choice is to keep it in place but stop using it from the active UI.

### Path strategy

The COS object key rules are:

- gallery: `gallery/<album>/<filename>`
- article: `articles/<yyyy>/<mm>/<filename>`

`<filename>` should be backend-generated, not taken directly from the user filename. The current timestamp-plus-UUID pattern already used by local uploads is good enough and should be reused.

Album names should be normalized into path-safe slugs before use in COS object keys.

### Pasted URL handling

PicGo URL support does not need a separate upload endpoint if the final save actions already accept URLs.

Rules:

- frontend can accept a pasted `http` or `https` URL;
- backend accepts the URL as part of draft save or publish payload;
- backend does not fetch or mirror that remote asset in this change;
- backend validates only basic URL shape where appropriate.

This keeps pasted PicGo links cheap and predictable.

## Frontend Design

### Owner API layer

Add a new service function:

- `uploadOwnerAsset(file, { kind, album })`

This replaces current owner image upload usage in the console.

### Gallery UI

The gallery screen should support two input paths:

- upload file
- paste existing image URL

For uploaded files, the UI sends:

- `kind=gallery`
- selected album

For pasted URLs, the UI stores and previews the URL directly without calling COS upload.

### Article UI

The article editor should support:

- upload article image to COS under `articles/<yyyy>/<mm>/...`
- paste an existing public image URL

The article draft or publish payload stores the final URL only. It should not depend on backend local file serving.

## Validation Rules

Backend validation:

- image extensions allowed: reuse the current allowed set unless there is a clear reason to expand it;
- MIME type must still resolve to `image/*`;
- `kind` must be explicit;
- `album` required for gallery and normalized before building the path.

Frontend validation:

- reject empty pasted URLs;
- reject non-HTTP URLs;
- show a clear error when COS upload is not configured.

## Testing

### Backend

Add controller tests for:

- owner-only access control on `/api/owner/assets`
- gallery upload success
- article upload success
- missing `album` for gallery
- invalid `kind`
- COS not configured
- upstream upload failure

Add uploader-level tests for:

- config parsing
- public URL generation
- object key generation for both asset kinds
- album slug normalization

Real COS network calls should not be part of unit tests.

### Frontend

Add service tests for:

- `uploadOwnerAsset` sends multipart data to `/api/owner/assets`
- `kind` and `album` are passed correctly

Add or extend owner console tests for:

- gallery upload path uses the new endpoint
- pasted URL path does not call file upload

## Deployment Impact

This change requires new secrets on the deployment side:

- `TENCENT_COS_SECRET_ID`
- `TENCENT_COS_SECRET_KEY`
- `TENCENT_COS_BUCKET`
- `TENCENT_COS_REGION`
- optional `TENCENT_COS_BASE_URL`

The deploy workflow and server env sync should be extended to carry these values to `/opt/acg-api/.env`, following the same pattern already used for owner publish GitHub settings.

## Implementation Notes

- Prefer the existing owner controller structure over introducing a new subsystem boundary unless tests become too awkward.
- Keep the uploader isolated enough to mock in tests.
- Do not mix pasted URL support with file upload handling in one opaque payload; file uploads should remain multipart and pasted URLs should remain plain JSON fields in draft/publish flows.

## Success Criteria

- Mobile can upload gallery and article images without PicGo.
- Desktop can continue using PicGo URLs directly.
- Uploaded assets return public COS URLs, not local backend file URLs.
- No COS secret value appears in committed code or frontend runtime data.
- Existing owner publish behavior continues to work.
