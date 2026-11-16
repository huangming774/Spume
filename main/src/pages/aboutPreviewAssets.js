// Historical compatibility: about-preview content used to embed absolute URLs
// on the former Tencent COS bucket. Those objects now live on Cloudflare R2,
// so residual old COS URLs are rewritten to the R2 direct origin.
const LEGACY_COS_ORIGIN =
  "https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/";
const R2_ASSET_ORIGIN = "https://fs.964643.xyz/";

export const rewriteAboutPreviewAssets = (markup = "") =>
  String(markup)
    .replaceAll(LEGACY_COS_ORIGIN, R2_ASSET_ORIGIN)
    .replace(/<img\b[^>]*>/gi, (tag) => {
      let next = tag;
      // Strip inline onerror handlers. Inside a shadow DOM an onerror
      // can fire on transient network hiccups and permanently hide the
      // image (display:none), making it look broken even though a retry
      // would succeed. Letting the broken-image placeholder show is
      // better than silently hiding it.
      next = next.replace(/\sonerror\s*=\s*"[^"]*"/gi, "");
      // Add referrerpolicy to avoid hotlink protection on the asset host.
      if (!/\sreferrerpolicy\s*=/i.test(next)) {
        next = next.replace(/<img\b/i, '<img referrerpolicy="no-referrer"');
      }
      if (!/\sdecoding\s*=/i.test(next)) {
        next = next.replace(/<img\b/i, '<img decoding="async"');
      }
      // All about-preview images are injected into a shadow DOM via
      // innerHTML. Browsers cannot reliably lazy-load <img> elements
      // inside shadow roots created this way - the IntersectionObserver
      // root may never see them, so they stay unloaded forever. The whole
      // page is only ~56 small SVG/JPG assets, so eager loading is cheap
      // and reliable.
      if (!/\sloading\s*=/i.test(next)) {
        next = next.replace(/<img\b/i, '<img loading="eager"');
      } else {
        next = next.replace(/\sloading\s*=\s*"[^"]*"/i, ' loading="eager"');
      }
      return next;
    });
