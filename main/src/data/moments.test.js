import test from "node:test";
import assert from "node:assert/strict";

import { moments } from "./moments.js";

test("starts with no glimmer notes", () => {
  assert.deepEqual(moments, []);
});
