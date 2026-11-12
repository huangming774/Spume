import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const storySource = readFileSync(new URL("./Story.jsx", import.meta.url), "utf8");
const cssSource = readFileSync(new URL("../index.css", import.meta.url), "utf8");

test("Story.jsx wires IntersectionObserver to open the envelope on view", () => {
  assert.match(storySource, /import clsx from "clsx"/);
  assert.match(storySource, /useEffect, useRef, useState/);
  assert.match(storySource, /IntersectionObserver/);
  assert.match(storySource, /setEnvelopeOpen\(true\)/);
  assert.match(storySource, /story-envelope-cta--open/);
});

test("Story.jsx uses editorial layout (meta number, serif title, light arc)", () => {
  assert.match(storySource, /aria-labelledby="story-envelope-heading"/);
  assert.match(storySource, /id="story-envelope-heading"/);
  assert.match(storySource, /story-envelope-meta/);
  assert.match(storySource, /story-envelope-title/);
  assert.match(storySource, /story-envelope-arc/);
  // letter format: salutation + sign-off make the "letter" intent explicit
  assert.match(storySource, /story-envelope-salutation/);
  assert.match(storySource, /story-envelope-signoff/);
  assert.match(storySource, /Dear visitor/);
  assert.match(storySource, /taozhiyo/);
  // dropped all the skeuomorphic envelope decor
  assert.doesNotMatch(storySource, /story-envelope-flap/);
  assert.doesNotMatch(storySource, /story-envelope-stamp/);
  assert.doesNotMatch(storySource, /story-envelope-postmark/);
  assert.doesNotMatch(storySource, /story-envelope-pocket/);
  assert.doesNotMatch(storySource, /story-envelope-barcode/);
});

test("envelope title uses Zentry serif (no inline art-letter highlights)", () => {
  assert.match(cssSource, /\.story-envelope-title\s*{[\s\S]*?font-family:\s*"Zentry"/);
  assert.doesNotMatch(storySource, /story-envelope-title[^>]*?<b>/);
  assert.doesNotMatch(cssSource, /\.story-envelope-title b\s*{/);
});

test("envelope is a soft frosted glass card with serif title", () => {
  assert.match(cssSource, /\.story-envelope-cta\s*{[\s\S]*?backdrop-filter:\s*blur/);
  assert.match(cssSource, /\.story-envelope-title\s*{[\s\S]*?font-family:\s*"Zentry"/);
});

test("envelope features the radial light arc as hero detail", () => {
  assert.match(cssSource, /\.story-envelope-arc\s*{[\s\S]*?radial-gradient/);
  assert.match(cssSource, /@keyframes storyEnvelopeArcBreath/);
});

test("envelope animates with gentle fade-in plus arc breathing — nothing mechanical", () => {
  assert.match(cssSource, /@keyframes storyEnvelopeFadeIn/);
  assert.match(
    cssSource,
    /\.story-envelope-cta--open\s*{[\s\S]*?animation:\s*storyEnvelopeFadeIn/
  );
  // no skeuomorphic flap/stamp animation
  assert.doesNotMatch(cssSource, /storyEnvelopeFlapOpen/);
  assert.doesNotMatch(cssSource, /storyEnvelopeStampStamp/);
});

test("envelope honors prefers-reduced-motion", () => {
  assert.match(
    cssSource,
    /@media \(prefers-reduced-motion: reduce\)\s*{[\s\S]*?\.story-envelope-cta[\s\S]*?animation:\s*none/
  );
});

test("envelope button has a visible focus ring (text-link style)", () => {
  assert.match(cssSource, /\.story-envelope-btn:focus-visible\s*{[^}]*outline:\s*2px solid #b76e79/s);
});
