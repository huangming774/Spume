export const EMOJI_CHOICES = [
  "🌈",
  "✨",
  "🎐",
  "🌸",
  "🍃",
  "☁️",
  "🌙",
  "⭐",
  "💫",
  "🫧",
  "🐾",
  "🍀",
  "🎀",
  "💌",
  "🧸",
  "🥰",
  "😊",
  "😭",
  "🥹",
  "🤍",
  "💛",
  "💙",
  "🔥",
  "👏",
];

export const appendEmojiToText = (value, emoji, maxLength) => {
  const text = value || "";
  const next = `${text}${emoji}`;

  if (typeof maxLength === "number" && next.length > maxLength) {
    return text;
  }

  return next;
};
