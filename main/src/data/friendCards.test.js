import test from "node:test";
import assert from "node:assert/strict";

import { friendCards } from "./friendCards.js";

test("starts with no friend links so the owner console can add them", () => {
  assert.deepEqual(friendCards, []);
});
