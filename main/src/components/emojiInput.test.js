import assert from "node:assert/strict";
import test from "node:test";

import { appendEmojiToText } from "./emojiInput.js";

test("appendEmojiToText appends emoji text", () => {
  assert.equal(appendEmojiToText("在校园，彩虹", "🌈", 300), "在校园，彩虹🌈");
});

test("appendEmojiToText keeps existing text when emoji would exceed maxLength", () => {
  assert.equal(appendEmojiToText("1234", "🌈", 5), "1234");
});

test("appendEmojiToText does not split an emoji when close to maxLength", () => {
  assert.equal(appendEmojiToText("123", "✨", 4), "123✨");
  assert.equal(appendEmojiToText("1234", "✨", 4), "1234");
});
