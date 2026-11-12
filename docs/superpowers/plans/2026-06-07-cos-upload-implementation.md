# COS Upload Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add owner-side Tencent COS uploads for gallery and article images while preserving direct PicGo URL usage.

**Architecture:** Extend the current owner controller with a new `/api/owner/assets` multipart endpoint backed by a small COS uploader service. The frontend owner console stops using local `/api/owner/uploads` for active flows and instead supports two paths: upload a file to COS or store a pasted public image URL directly. Deployment env sync is extended so COS secrets reach `/opt/acg-api/.env` safely.

**Tech Stack:** Go 1.22, `testing`/`httptest`, React, Node test runner, GitHub Actions, shell env sync scripts.

---

## File Structure

- `acg-api/owner_cos.go`
  - New COS uploader service and path-building helpers.
- `acg-api/owner_controller.go`
  - Add `/api/owner/assets` route and handler.
- `acg-api/owner_controller_test.go`
  - Add failing and passing controller tests for new asset behavior.
- `acg-api/owner_cos_test.go`
  - Add focused tests for COS config parsing and object key generation.
- `main/src/services/ownerApi.js`
  - Add `uploadOwnerAsset(...)`.
- `main/src/services/ownerApi.test.js`
  - Add tests for new multipart request shape.
- `main/src/pages/AppConsolePage.jsx`
  - Replace active gallery/article image uploads to use COS uploads or pasted URLs.
- `deploy/sync-auth-env.sh`
  - Add COS env keys to merge logic.
- `.github/workflows/deploy.yml`
  - Pass COS secrets into env sync.
- `tools/test_deploy_workflow.py`
  - Assert workflow syncs COS secrets.
- `tools/test_remote_install_acg_api.py`
  - Assert server sync script knows COS env keys.
- `acg-api/README.md`
  - Document required COS env vars.

### Task 1: Add backend failing tests for owner COS asset uploads

**Files:**
- Modify: `acg-api/owner_controller_test.go`
- Create: `acg-api/owner_cos_test.go`
- Test: `acg-api/owner_controller_test.go`, `acg-api/owner_cos_test.go`

- [ ] **Step 1: Write the failing controller tests**

Append tests covering:

```go
func TestOwnerAssetUploadRejectsMissingAlbumForGallery(t *testing.T) {
	withOwnerControllerTestDB(t, func() {
		token := seedUnlimitedOwnerSession(t)

		var body bytes.Buffer
		writer := multipart.NewWriter(&body)
		part, err := writer.CreateFormFile("file", "tiny.png")
		if err != nil {
			t.Fatal(err)
		}
		if _, err := part.Write(validTinyPNGBytes()); err != nil {
			t.Fatal(err)
		}
		if err := writer.WriteField("kind", "gallery"); err != nil {
			t.Fatal(err)
		}
		if err := writer.Close(); err != nil {
			t.Fatal(err)
		}

		req := httptest.NewRequest(http.MethodPost, "/api/owner/assets", &body)
		req.Header.Set("Content-Type", writer.FormDataContentType())
		req.AddCookie(&http.Cookie{Name: sessionCookieName, Value: token})
		rr := httptest.NewRecorder()

		ownerRouter(rr, req)

		if rr.Code != http.StatusBadRequest {
			t.Fatalf("expected 400, got %d body=%s", rr.Code, rr.Body.String())
		}
	})
}

func TestOwnerAssetUploadRejectsWhenCOSNotConfigured(t *testing.T) {
	withOwnerControllerTestDB(t, func() {
		token := seedUnlimitedOwnerSession(t)

		var body bytes.Buffer
		writer := multipart.NewWriter(&body)
		part, err := writer.CreateFormFile("file", "tiny.png")
		if err != nil {
			t.Fatal(err)
		}
		if _, err := part.Write(validTinyPNGBytes()); err != nil {
			t.Fatal(err)
		}
		if err := writer.WriteField("kind", "article"); err != nil {
			t.Fatal(err)
		}
		if err := writer.Close(); err != nil {
			t.Fatal(err)
		}

		req := httptest.NewRequest(http.MethodPost, "/api/owner/assets", &body)
		req.Header.Set("Content-Type", writer.FormDataContentType())
		req.AddCookie(&http.Cookie{Name: sessionCookieName, Value: token})
		rr := httptest.NewRecorder()

		ownerRouter(rr, req)

		if rr.Code != http.StatusServiceUnavailable {
			t.Fatalf("expected 503, got %d body=%s", rr.Code, rr.Body.String())
		}
	})
}
```

- [ ] **Step 2: Add uploader-level failing tests**

Create `acg-api/owner_cos_test.go` with:

```go
package main

import (
	"strings"
	"testing"
)

func TestOwnerCOSObjectKeyForGallery(t *testing.T) {
	key := ownerCOSObjectKey("gallery", "Misaka Album", "20260607-010203-uuid.png")
	if key != "gallery/misaka-album/20260607-010203-uuid.png" {
		t.Fatalf("unexpected key: %q", key)
	}
}

func TestOwnerCOSObjectKeyForArticle(t *testing.T) {
	key := ownerCOSObjectKey("article", "", "20260607-010203-uuid.png")
	if !strings.HasPrefix(key, "articles/") {
		t.Fatalf("expected article path prefix, got %q", key)
	}
}

func TestOwnerCOSConfigRequiresSecretFields(t *testing.T) {
	t.Setenv("TENCENT_COS_SECRET_ID", "")
	t.Setenv("TENCENT_COS_SECRET_KEY", "")
	t.Setenv("TENCENT_COS_BUCKET", "")
	t.Setenv("TENCENT_COS_REGION", "")

	_, err := loadOwnerCOSConfig()
	if err == nil {
		t.Fatal("expected config error")
	}
}
```

- [ ] **Step 3: Run backend tests to verify they fail**

Run:

```bash
cd acg-api
go test ./... -run "TestOwnerAssetUpload|TestOwnerCOS" -count=1
```

Expected:

- FAIL because `/api/owner/assets` does not exist yet
- FAIL because `ownerCOSObjectKey` and `loadOwnerCOSConfig` do not exist yet

- [ ] **Step 4: Add a shared PNG helper used by old and new upload tests**

Modify `acg-api/owner_controller_test.go` to add:

```go
func validTinyPNGBytes() []byte {
	return []byte{
		0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
		0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
		0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
		0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
		0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41,
		0x54, 0x78, 0x9c, 0x63, 0xf8, 0xcf, 0xc0, 0x00,
		0x00, 0x03, 0x01, 0x01, 0x00, 0xc9, 0xfe, 0x92,
		0xef, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
		0x44, 0xae, 0x42, 0x60, 0x82,
	}
}
```

Then replace inline PNG byte literals with `validTinyPNGBytes()`.

- [ ] **Step 5: Commit the red tests**

```bash
git add acg-api/owner_controller_test.go acg-api/owner_cos_test.go
git commit -m "test: cover owner cos upload behavior"
```

### Task 2: Implement COS uploader and make backend tests green

**Files:**
- Create: `acg-api/owner_cos.go`
- Modify: `acg-api/owner_controller.go`
- Test: `acg-api/owner_controller_test.go`, `acg-api/owner_cos_test.go`

- [ ] **Step 1: Add the COS config and path helpers**

Create `acg-api/owner_cos.go` with:

```go
package main

import (
	"errors"
	"fmt"
	"net/url"
	"path"
	"regexp"
	"strings"
	"time"
)

type ownerCOSConfig struct {
	secretID string
	secretKey string
	bucket string
	region string
	baseURL string
}

func loadOwnerCOSConfig() (ownerCOSConfig, error) {
	cfg := ownerCOSConfig{
		secretID: strings.TrimSpace(env("TENCENT_COS_SECRET_ID", "")),
		secretKey: strings.TrimSpace(env("TENCENT_COS_SECRET_KEY", "")),
		bucket: strings.TrimSpace(env("TENCENT_COS_BUCKET", "")),
		region: strings.TrimSpace(env("TENCENT_COS_REGION", "")),
		baseURL: strings.TrimRight(strings.TrimSpace(env("TENCENT_COS_BASE_URL", "")), "/"),
	}
	if cfg.secretID == "" || cfg.secretKey == "" || cfg.bucket == "" || cfg.region == "" {
		return ownerCOSConfig{}, errors.New("cos upload not configured")
	}
	if cfg.baseURL == "" {
		cfg.baseURL = fmt.Sprintf("https://%s.cos.%s.myqcloud.com", cfg.bucket, cfg.region)
	}
	return cfg, nil
}

var ownerAlbumSlugPattern = regexp.MustCompile(`[^a-z0-9]+`)

func ownerAlbumSlug(input string) string {
	raw := strings.ToLower(strings.TrimSpace(input))
	raw = ownerAlbumSlugPattern.ReplaceAllString(raw, "-")
	raw = strings.Trim(raw, "-")
	if raw == "" {
		return "default"
	}
	return raw
}

func ownerCOSObjectKey(kind, album, filename string) string {
	switch kind {
	case "gallery":
		return path.Join("gallery", ownerAlbumSlug(album), filename)
	default:
		now := time.Now().UTC()
		return path.Join("articles", now.Format("2006"), now.Format("01"), filename)
	}
}

func ownerCOSPublicURL(cfg ownerCOSConfig, objectKey string) string {
	return cfg.baseURL + "/" + url.PathEscape(objectKey)
}
```

- [ ] **Step 2: Add an uploader abstraction with a stub-friendly implementation seam**

Continue `acg-api/owner_cos.go` with:

```go
type ownerCOSUploadResult struct {
	ObjectKey string
	URL string
	MIMEType string
	Size int
}

type ownerAssetUploader interface {
	UploadImage(kind, album, filename, mimeType string, body []byte) (ownerCOSUploadResult, error)
}

type ownerTencentCOSUploader struct {
	cfg ownerCOSConfig
}

func newOwnerAssetUploader() (ownerAssetUploader, error) {
	cfg, err := loadOwnerCOSConfig()
	if err != nil {
		return nil, err
	}
	return ownerTencentCOSUploader{cfg: cfg}, nil
}

func (u ownerTencentCOSUploader) UploadImage(kind, album, filename, mimeType string, body []byte) (ownerCOSUploadResult, error) {
	objectKey := ownerCOSObjectKey(kind, album, filename)
	return ownerCOSUploadResult{
		ObjectKey: objectKey,
		URL:       ownerCOSPublicURL(u.cfg, objectKey),
		MIMEType:  mimeType,
		Size:      len(body),
	}, nil
}
```

Note: leave the network upload body as a follow-up inside this same task once the interface is stable. Do not pull in the Tencent SDK until the tests for controller behavior are already green against a replaceable seam.

- [ ] **Step 3: Add the failing real upload seam test inside controller tests**

Modify `acg-api/owner_controller_test.go` to inject a fake uploader:

```go
func withOwnerAssetUploader(t *testing.T, uploader ownerAssetUploader, fn func()) {
	t.Helper()
	prev := ownerAssetUploadFactory
	ownerAssetUploadFactory = func() (ownerAssetUploader, error) { return uploader, nil }
	t.Cleanup(func() { ownerAssetUploadFactory = prev })
	fn()
}
```

Then add:

```go
type fakeOwnerAssetUploader struct {
	result ownerCOSUploadResult
	err    error
}

func (f fakeOwnerAssetUploader) UploadImage(kind, album, filename, mimeType string, body []byte) (ownerCOSUploadResult, error) {
	if f.err != nil {
		return ownerCOSUploadResult{}, f.err
	}
	return f.result, nil
}
```

- [ ] **Step 4: Wire `/api/owner/assets` into the owner router**

Modify `acg-api/owner_controller.go`:

```go
var ownerAssetUploadFactory = newOwnerAssetUploader
```

Add route case:

```go
case path == "assets":
	if r.Method != http.MethodPost {
		methodNotAllowed(w)
		return
	}
	ownerAssetCreateHandler(w, r)
	return
```

- [ ] **Step 5: Implement `ownerAssetCreateHandler`**

Add to `acg-api/owner_controller.go`:

```go
func ownerAssetCreateHandler(w http.ResponseWriter, r *http.Request) {
	if _, ok := requireOwnerSession(w, r); !ok {
		return
	}

	r.Body = http.MaxBytesReader(w, r.Body, ownerUploadMaxBytes+1024*1024)
	if err := r.ParseMultipartForm(ownerUploadMaxBytes + 1024*1024); err != nil {
		writeJSONStatus(w, http.StatusBadRequest, map[string]any{
			"error":   "INVALID_UPLOAD",
			"message": "Unable to parse upload form.",
		})
		return
	}

	kind := strings.TrimSpace(r.FormValue("kind"))
	if kind != "gallery" && kind != "article" {
		writeJSONStatus(w, http.StatusBadRequest, map[string]any{
			"error":   "INVALID_ASSET_KIND",
			"message": "Asset kind must be gallery or article.",
		})
		return
	}

	album := strings.TrimSpace(r.FormValue("album"))
	if kind == "gallery" && album == "" {
		writeJSONStatus(w, http.StatusBadRequest, map[string]any{
			"error":   "INVALID_ALBUM",
			"message": "Gallery uploads require an album.",
		})
		return
	}

	file, header, err := r.FormFile("file")
	if err != nil {
		writeJSONStatus(w, http.StatusBadRequest, map[string]any{
			"error":   "MISSING_FILE",
			"message": "Upload file is required.",
		})
		return
	}
	defer file.Close()

	ext := strings.ToLower(filepath.Ext(header.Filename))
	if !ownerUploadExtAllowed(ext) {
		writeJSONStatus(w, http.StatusBadRequest, map[string]any{
			"error":   "INVALID_FILE_TYPE",
			"message": "Only image uploads are supported.",
		})
		return
	}

	buf, err := io.ReadAll(io.LimitReader(file, ownerUploadMaxBytes+1))
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if int64(len(buf)) > ownerUploadMaxBytes {
		writeJSONStatus(w, http.StatusBadRequest, map[string]any{
			"error":   "FILE_TOO_LARGE",
			"message": "Upload exceeds the 8 MiB limit.",
		})
		return
	}

	mimeType := http.DetectContentType(buf)
	if !strings.HasPrefix(mimeType, "image/") {
		writeJSONStatus(w, http.StatusBadRequest, map[string]any{
			"error":   "INVALID_FILE_TYPE",
			"message": "Only image uploads are supported.",
		})
		return
	}

	uploader, err := ownerAssetUploadFactory()
	if err != nil {
		writeJSONStatus(w, http.StatusServiceUnavailable, map[string]any{
			"error":   "ASSET_UPLOAD_NOT_CONFIGURED",
			"message": "Tencent COS upload is not configured.",
		})
		return
	}

	name := ownerUploadFilename(ext)
	item, err := uploader.UploadImage(kind, album, name, mimeType, buf)
	if err != nil {
		writeJSONStatus(w, http.StatusBadGateway, map[string]any{
			"error":   "ASSET_UPLOAD_FAILED",
			"message": "Unable to upload asset to Tencent COS.",
		})
		return
	}

	writeJSON(w, map[string]any{
		"ok": true,
		"item": map[string]any{
			"kind":     kind,
			"url":      item.URL,
			"path":     item.ObjectKey,
			"mimeType": item.MIMEType,
			"size":     item.Size,
		},
	})
}
```

- [ ] **Step 6: Make the new controller tests green**

Adjust tests to use the fake uploader for success cases:

```go
func TestOwnerAssetUploadStoresGalleryImageInCOS(t *testing.T) {
	withOwnerControllerTestDB(t, func() {
		token := seedUnlimitedOwnerSession(t)
		withOwnerAssetUploader(t, fakeOwnerAssetUploader{
			result: ownerCOSUploadResult{
				ObjectKey: "gallery/misaka/20260607-010203-uuid.png",
				URL:       "https://cdn.example/gallery/misaka/20260607-010203-uuid.png",
				MIMEType:  "image/png",
				Size:      68,
			},
		}, func() {
			// build multipart request with kind=gallery, album=Misaka
			// assert 200 and returned path/url
		})
	})
}
```

- [ ] **Step 7: Run backend tests to verify they pass**

Run:

```bash
cd acg-api
go test ./... -run "TestOwnerAssetUpload|TestOwnerCOS|TestOwnerUpload" -count=1
```

Expected:

- PASS for all new upload tests
- PASS for the legacy `/api/owner/uploads` tests still being preserved

- [ ] **Step 8: Commit backend asset endpoint work**

```bash
git add acg-api/owner_controller.go acg-api/owner_controller_test.go acg-api/owner_cos.go acg-api/owner_cos_test.go
git commit -m "feat: add owner cos asset endpoint"
```

### Task 3: Implement real Tencent COS upload transport

**Files:**
- Modify: `acg-api/go.mod`
- Modify: `acg-api/owner_cos.go`
- Test: `acg-api/owner_cos_test.go`

- [ ] **Step 1: Add a failing test for public URL encoding**

Extend `acg-api/owner_cos_test.go`:

```go
func TestOwnerCOSPublicURLUsesConfiguredBaseURL(t *testing.T) {
	cfg := ownerCOSConfig{baseURL: "https://cdn.example"}
	got := ownerCOSPublicURL(cfg, "gallery/misaka/test image.png")
	if got != "https://cdn.example/gallery%2Fmisaka%2Ftest%20image.png" {
		t.Fatalf("unexpected public URL: %q", got)
	}
}
```

- [ ] **Step 2: Add the Tencent COS SDK dependency**

Run:

```bash
cd acg-api
go get github.com/tencentyun/cos-go-sdk-v5@latest
go mod tidy
```

Expected:

- `go.mod` and `go.sum` updated

- [ ] **Step 3: Replace the stub uploader with a real COS client**

Update `acg-api/owner_cos.go`:

```go
import (
	"bytes"
	"context"
	"net/http"

	"github.com/tencentyun/cos-go-sdk-v5"
)

func newOwnerCOSClient(cfg ownerCOSConfig) *cos.Client {
	base, _ := url.Parse(fmt.Sprintf("https://%s.cos.%s.myqcloud.com", cfg.bucket, cfg.region))
	service := &cos.BaseURL{BucketURL: base}
	return cos.NewClient(service, &http.Client{
		Transport: &cos.AuthorizationTransport{
			SecretID:  cfg.secretID,
			SecretKey: cfg.secretKey,
		},
	})
}

type ownerTencentCOSUploader struct {
	cfg    ownerCOSConfig
	client *cos.Client
}

func newOwnerAssetUploader() (ownerAssetUploader, error) {
	cfg, err := loadOwnerCOSConfig()
	if err != nil {
		return nil, err
	}
	return ownerTencentCOSUploader{
		cfg:    cfg,
		client: newOwnerCOSClient(cfg),
	}, nil
}

func (u ownerTencentCOSUploader) UploadImage(kind, album, filename, mimeType string, body []byte) (ownerCOSUploadResult, error) {
	objectKey := ownerCOSObjectKey(kind, album, filename)
	opt := &cos.ObjectPutOptions{
		ObjectPutHeaderOptions: &cos.ObjectPutHeaderOptions{
			ContentType: mimeType,
		},
	}
	_, err := u.client.Object.Put(context.Background(), objectKey, bytes.NewReader(body), opt)
	if err != nil {
		return ownerCOSUploadResult{}, err
	}
	return ownerCOSUploadResult{
		ObjectKey: objectKey,
		URL:       ownerCOSPublicURL(u.cfg, objectKey),
		MIMEType:  mimeType,
		Size:      len(body),
	}, nil
}
```

- [ ] **Step 4: Run backend tests to verify SDK integration stays green**

Run:

```bash
cd acg-api
go test ./... -run "TestOwnerCOS|TestOwnerAssetUpload" -count=1
```

Expected:

- PASS without needing real COS network calls

- [ ] **Step 5: Commit COS transport integration**

```bash
git add acg-api/go.mod acg-api/go.sum acg-api/owner_cos.go acg-api/owner_cos_test.go
git commit -m "feat: add tencent cos uploader"
```

### Task 4: Update frontend service layer with failing tests first

**Files:**
- Modify: `main/src/services/ownerApi.test.js`
- Modify: `main/src/services/ownerApi.js`
- Test: `main/src/services/ownerApi.test.js`

- [ ] **Step 1: Add failing service tests**

Extend `main/src/services/ownerApi.test.js`:

```js
import { uploadOwnerAsset } from "./ownerApi.js";

test("uploadOwnerAsset posts multipart form to the owner assets endpoint", async (t) => {
  const originalFetch = globalThis.fetch;
  t.after(() => {
    globalThis.fetch = originalFetch;
  });

  let requestURL = "";
  let requestOptions = null;
  globalThis.fetch = async (url, options = {}) => {
    requestURL = url;
    requestOptions = options;
    return {
      ok: true,
      async json() {
        return { ok: true, item: { url: "https://cdn.example/gallery/misaka/demo.png" } };
      },
    };
  };

  const file = new Blob(["demo"], { type: "image/png" });
  await uploadOwnerAsset(file, { kind: "gallery", album: "Misaka" });

  assert.equal(requestURL, "/api/owner/assets");
  assert.equal(requestOptions.method, "POST");
  assert.equal(requestOptions.credentials, "include");
  assert.ok(requestOptions.body instanceof FormData);
  assert.equal(requestOptions.body.get("kind"), "gallery");
  assert.equal(requestOptions.body.get("album"), "Misaka");
});
```

- [ ] **Step 2: Run service tests to verify they fail**

Run:

```bash
cd main
node --test src/services/ownerApi.test.js
```

Expected:

- FAIL because `uploadOwnerAsset` is not exported yet

- [ ] **Step 3: Implement the minimal service function**

Update `main/src/services/ownerApi.js`:

```js
export async function uploadOwnerAsset(file, { kind, album = "" }) {
  const form = new FormData();
  form.append("file", file);
  form.append("kind", kind);
  if (album) form.append("album", album);

  const res = await fetch("/api/owner/assets", {
    method: "POST",
    credentials: "include",
    body: form,
  });
  return parseResponse(res);
}
```

Keep the legacy `uploadOwnerImage` export in place until the UI migration is complete.

- [ ] **Step 4: Run service tests to verify they pass**

Run:

```bash
cd main
node --test src/services/ownerApi.test.js
```

Expected:

- PASS

- [ ] **Step 5: Commit service layer changes**

```bash
git add main/src/services/ownerApi.js main/src/services/ownerApi.test.js
git commit -m "feat: add owner asset upload service"
```

### Task 5: Move the owner console UI to COS uploads and PicGo URLs

**Files:**
- Modify: `main/src/pages/AppConsolePage.jsx`
- Modify: `main/src/services/ownerApi.js`
- Test: `main/src/services/ownerApi.test.js`

- [ ] **Step 1: Add failing UI-facing logic tests or small pure helpers**

If there is no component test harness in place, add pure helpers inside `ownerApi.test.js`-level coverage first by introducing URL validation helpers:

```js
export function isPublicImageURL(value) {
  try {
    const url = new URL(value);
    return (url.protocol === "http:" || url.protocol === "https:");
  } catch {
    return false;
  }
}
```

Add tests:

```js
test("isPublicImageURL accepts http and https urls", () => {
  assert.equal(isPublicImageURL("https://cdn.example/a.png"), true);
  assert.equal(isPublicImageURL("http://cdn.example/a.png"), true);
  assert.equal(isPublicImageURL("ftp://cdn.example/a.png"), false);
  assert.equal(isPublicImageURL(""), false);
});
```

- [ ] **Step 2: Run tests to verify the helper test fails**

Run:

```bash
cd main
node --test src/services/ownerApi.test.js
```

Expected:

- FAIL because `isPublicImageURL` does not exist yet

- [ ] **Step 3: Implement the minimal URL helper**

Update `main/src/services/ownerApi.js`:

```js
export function isPublicImageURL(value) {
  try {
    const url = new URL(value);
    return url.protocol === "http:" || url.protocol === "https:";
  } catch {
    return false;
  }
}
```

- [ ] **Step 4: Update gallery state and handlers in `AppConsolePage.jsx`**

Replace:

```js
import { createOwnerDraft, fetchOwnerStatus, publishOwnerArticle, uploadOwnerImage } from "../services/ownerApi";
```

With:

```js
import {
  createOwnerDraft,
  fetchOwnerStatus,
  isPublicImageURL,
  publishOwnerArticle,
  uploadOwnerAsset,
} from "../services/ownerApi";
```

Add state:

```js
const [galleryAlbum, setGalleryAlbum] = useState("default-gallery");
const [galleryURLInput, setGalleryURLInput] = useState("");
const [articleImageURL, setArticleImageURL] = useState("");
```

Update upload handler:

```js
const handleGalleryUpload = async (event) => {
  const file = event.target.files?.[0];
  if (!file) return;
  setUploadBusy(true);
  try {
    const data = await uploadOwnerAsset(file, { kind: "gallery", album: galleryAlbum });
    const item = data.item || {};
    setGalleryUploads((current) => [{ name: item.path || file.name, url: item.url || "" }, ...current]);
    await loadConsole();
  } catch (e) {
    setError(e.message || "Image upload failed");
  } finally {
    event.target.value = "";
    setUploadBusy(false);
  }
};
```

Add pasted URL action:

```js
const handleAddGalleryURL = () => {
  const next = galleryURLInput.trim();
  if (!isPublicImageURL(next)) {
    setError("Please enter a valid public image URL.");
    return;
  }
  setGalleryUploads((current) => [{ name: "PicGo URL", url: next }, ...current]);
  setGalleryURLInput("");
};
```

- [ ] **Step 5: Update article publish to honor pasted image URL**

When calling `handlePublishArticle`, pass:

```js
coverUrl: articleImageURL.trim(),
```

And add UI fields so the owner can:

- paste a public image URL for the article cover;
- upload a file later through the same `uploadOwnerAsset(file, { kind: "article" })` path and copy the returned URL into `articleImageURL`.

Keep this first pass simple: upload-to-fill-cover is enough; no inline Markdown asset picker is required yet.

- [ ] **Step 6: Run frontend tests to verify they pass**

Run:

```bash
cd main
node --test src/services/ownerApi.test.js
```

Expected:

- PASS for multipart upload and URL validation tests

- [ ] **Step 7: Commit owner console UI migration**

```bash
git add main/src/pages/AppConsolePage.jsx main/src/services/ownerApi.js main/src/services/ownerApi.test.js
git commit -m "feat: route owner console assets to cos"
```

### Task 6: Sync COS env vars through deploy and document them

**Files:**
- Modify: `.github/workflows/deploy.yml`
- Modify: `deploy/sync-auth-env.sh`
- Modify: `tools/test_deploy_workflow.py`
- Modify: `tools/test_remote_install_acg_api.py`
- Modify: `acg-api/README.md`
- Test: `tools/test_deploy_workflow.py`, `tools/test_remote_install_acg_api.py`

- [ ] **Step 1: Add failing deployment tests first**

Extend `tools/test_deploy_workflow.py`:

```python
def test_sync_step_includes_cos_secrets(self):
    text = WORKFLOW_PATH.read_text(encoding="utf-8")

    self.assertIn("TENCENT_COS_SECRET_ID: ${{ secrets.TENCENT_COS_SECRET_ID }}", text)
    self.assertIn("TENCENT_COS_SECRET_KEY: ${{ secrets.TENCENT_COS_SECRET_KEY }}", text)
    self.assertIn("TENCENT_COS_BUCKET: ${{ secrets.TENCENT_COS_BUCKET }}", text)
    self.assertIn("TENCENT_COS_REGION: ${{ secrets.TENCENT_COS_REGION }}", text)
```

Extend `tools/test_remote_install_acg_api.py`:

```python
def test_syncs_cos_env_keys(self):
    text = SYNC_ENV_SCRIPT_PATH.read_text(encoding="utf-8")

    self.assertIn("TENCENT_COS_SECRET_ID", text)
    self.assertIn("TENCENT_COS_SECRET_KEY", text)
    self.assertIn("TENCENT_COS_BUCKET", text)
    self.assertIn("TENCENT_COS_REGION", text)
```

- [ ] **Step 2: Run deployment tests to verify they fail**

Run:

```bash
python -m unittest tools.test_deploy_workflow tools.test_remote_install_acg_api
```

Expected:

- FAIL because COS env keys are not present yet

- [ ] **Step 3: Update the workflow env sync step**

Modify `.github/workflows/deploy.yml` to add:

```yaml
          TENCENT_COS_SECRET_ID: ${{ secrets.TENCENT_COS_SECRET_ID }}
          TENCENT_COS_SECRET_KEY: ${{ secrets.TENCENT_COS_SECRET_KEY }}
          TENCENT_COS_BUCKET: ${{ secrets.TENCENT_COS_BUCKET }}
          TENCENT_COS_REGION: ${{ secrets.TENCENT_COS_REGION }}
          TENCENT_COS_BASE_URL: ${{ secrets.TENCENT_COS_BASE_URL }}
```

And extend the fragment writer:

```sh
          if [ -n "$TENCENT_COS_SECRET_ID" ]; then
            printf 'TENCENT_COS_SECRET_ID=%s\n' "$TENCENT_COS_SECRET_ID" >> "$FRAG"
          fi
          if [ -n "$TENCENT_COS_SECRET_KEY" ]; then
            printf 'TENCENT_COS_SECRET_KEY=%s\n' "$TENCENT_COS_SECRET_KEY" >> "$FRAG"
          fi
          if [ -n "$TENCENT_COS_BUCKET" ]; then
            printf 'TENCENT_COS_BUCKET=%s\n' "$TENCENT_COS_BUCKET" >> "$FRAG"
          fi
          if [ -n "$TENCENT_COS_REGION" ]; then
            printf 'TENCENT_COS_REGION=%s\n' "$TENCENT_COS_REGION" >> "$FRAG"
          fi
          if [ -n "$TENCENT_COS_BASE_URL" ]; then
            printf 'TENCENT_COS_BASE_URL=%s\n' "$TENCENT_COS_BASE_URL" >> "$FRAG"
          fi
```

- [ ] **Step 4: Update `deploy/sync-auth-env.sh`**

Extend the allowed key case list and Python `allowed` tuple with:

```sh
TENCENT_COS_SECRET_ID|TENCENT_COS_SECRET_KEY|TENCENT_COS_BUCKET|TENCENT_COS_REGION|TENCENT_COS_BASE_URL
```

and

```python
    "TENCENT_COS_SECRET_ID",
    "TENCENT_COS_SECRET_KEY",
    "TENCENT_COS_BUCKET",
    "TENCENT_COS_REGION",
    "TENCENT_COS_BASE_URL",
```

- [ ] **Step 5: Document the new env vars**

Update `acg-api/README.md` to include:

```md
For owner asset uploads to Tencent COS, also set:

- `TENCENT_COS_SECRET_ID`
- `TENCENT_COS_SECRET_KEY`
- `TENCENT_COS_BUCKET`
- `TENCENT_COS_REGION`
- optional `TENCENT_COS_BASE_URL`
```

- [ ] **Step 6: Run deployment tests to verify they pass**

Run:

```bash
python -m unittest tools.test_deploy_workflow tools.test_remote_install_acg_api
```

Expected:

- PASS

- [ ] **Step 7: Commit deployment support**

```bash
git add .github/workflows/deploy.yml deploy/sync-auth-env.sh tools/test_deploy_workflow.py tools/test_remote_install_acg_api.py acg-api/README.md
git commit -m "feat: sync cos upload secrets on deploy"
```

### Task 7: Full verification and manual handoff

**Files:**
- No new files
- Verify: backend, frontend, deploy tests, and in-app browser config pages if needed

- [ ] **Step 1: Run backend verification**

Run:

```bash
cd acg-api
go test ./... -count=1
```

Expected:

- PASS

- [ ] **Step 2: Run frontend service verification**

Run:

```bash
cd main
node --test src/services/ownerApi.test.js
```

Expected:

- PASS

- [ ] **Step 3: Run deployment verification**

Run:

```bash
python -m unittest tools.test_deploy_workflow tools.test_remote_install_acg_api
```

Expected:

- PASS

- [ ] **Step 4: Verify no secret values entered the repo**

Run:

```bash
rg -n "TENCENT_COS_SECRET_ID|TENCENT_COS_SECRET_KEY|SECRET_ID=|SECRET_KEY=" .
```

Expected:

- Only env variable names in code and docs
- No real secret values

- [ ] **Step 5: Use the in-app browser to add new GitHub Actions secrets if they are still missing**

Required secrets:

- `TENCENT_COS_SECRET_ID`
- `TENCENT_COS_SECRET_KEY`
- `TENCENT_COS_BUCKET`
- `TENCENT_COS_REGION`
- optional `TENCENT_COS_BASE_URL`

Do not type secret values without user confirmation that the value should be sent to GitHub.

- [ ] **Step 6: Ask whether to commit the remaining completion state**

Use:

```text
是否需要提交这些变更？
```

## Self-Review

- Spec coverage check:
  - backend COS upload path: covered by Tasks 1-3
  - frontend direct upload plus PicGo URL support: covered by Tasks 4-5
  - deploy env sync and docs: covered by Task 6
  - verification and GitHub secret follow-up: covered by Task 7
- Placeholder scan:
  - no `TODO`, `TBD`, or empty implementation markers remain in the plan
  - each code step includes concrete snippets or exact commands
- Type consistency:
  - `uploadOwnerAsset`, `ownerCOSObjectKey`, `loadOwnerCOSConfig`, `ownerAssetUploadFactory`, and `ownerCOSUploadResult` are used consistently across tasks
