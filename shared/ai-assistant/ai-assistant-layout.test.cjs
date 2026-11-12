const test = require("node:test");
const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { resolve } = require("node:path");

const cssSource = readFileSync(resolve(__dirname, "ai-assistant.css"), "utf8");

test("keeps the global AI assistant usable within phone safe areas", () => {
  const mobileBlock = cssSource.match(
    /@media \(max-width:\s*768px\)\s*\{(?<rules>[\s\S]*)\n\}/,
  );

  assert.ok(mobileBlock, "missing global AI assistant mobile media block");
  assert.match(
    mobileBlock.groups.rules,
    /\.bai-overlay\s*\{[\s\S]*?padding:\s*max\(8px,\s*env\(safe-area-inset-top,\s*0px\)\) 8px max\(8px,\s*env\(safe-area-inset-bottom,\s*0px\)\)/,
  );
  assert.match(
    mobileBlock.groups.rules,
    /\.bai-app\s*\{[\s\S]*?width:\s*min\(100%,\s*calc\(100vw - 16px\)\)/,
  );
  assert.match(
    mobileBlock.groups.rules,
    /\.bai-app\s*\{[\s\S]*?height:\s*min\(100dvh,\s*calc\(100dvh - 16px - env\(safe-area-inset-top,\s*0px\) - env\(safe-area-inset-bottom,\s*0px\)\)\)/,
  );
  assert.match(
    mobileBlock.groups.rules,
    /\.bai-app\s*\{[\s\S]*?border-radius:\s*18px/,
  );
});

test("keeps global AI assistant controls from pushing past phone width", () => {
  const mobileBlock = cssSource.match(
    /@media \(max-width:\s*768px\)\s*\{(?<rules>[\s\S]*)\n\}/,
  );

  assert.ok(mobileBlock, "missing global AI assistant mobile media block");
  assert.match(
    mobileBlock.groups.rules,
    /\.bai-aside\s*\{[\s\S]*?max-width:\s*100%[\s\S]*?overflow-x:\s*auto/,
  );
  assert.match(
    mobileBlock.groups.rules,
    /\.blog-ai-mode-tabs\s*\{[\s\S]*?flex:\s*0 0 auto/,
  );
  assert.match(
    mobileBlock.groups.rules,
    /\.bai-composer-shell\s*\{[\s\S]*?display:\s*grid[\s\S]*?grid-template-columns:\s*minmax\(0,\s*1fr\) auto/,
  );
  assert.match(
    mobileBlock.groups.rules,
    /#blog-ai-send\s*\{[\s\S]*?min-width:\s*4\.25rem/,
  );
});
