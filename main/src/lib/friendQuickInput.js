const imageExtensions = /\.(avif|gif|ico|jpe?g|png|svg|webp)(?:[?#].*)?$/i;

function cleanLine(line) {
  return line.replace(/\s+/g, " ").trim();
}

function splitLabel(line) {
  if (parseURL(line)) return { label: "", value: line };
  const match = line.match(/^([^:：]{1,12})[:：]\s*(.+)$/);
  if (!match) return { label: "", value: line };
  return {
    label: match[1].trim().toLowerCase(),
    value: match[2].trim(),
  };
}

function parseURL(value) {
  try {
    const parsed = new URL(value);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return null;
    return parsed;
  } catch {
    return null;
  }
}

function isAvatarURL(value) {
  const parsed = parseURL(value);
  if (!parsed) return false;
  const path = `${parsed.pathname}${parsed.search}`.toLowerCase();
  return imageExtensions.test(path) || /avatar|head|logo|icon|favicon|uploads?/.test(path);
}

function assignLabel(target, label, value) {
  if (/^(name|名称|站名|站点|网站|博客)$/.test(label)) {
    target.name = value;
    return true;
  }
  if (/^(desc|description|描述|简介|介绍|说明)$/.test(label)) {
    target.desc = value;
    return true;
  }
  if (/^(url|link|链接|地址|网址|站点地址|网站地址)$/.test(label)) {
    target.url = value;
    return true;
  }
  if (/^(avatar|头像|图标|logo|图片)$/.test(label)) {
    target.avatar = value;
    return true;
  }
  return false;
}

function nameScore(value) {
  let score = 0;
  if (/[。.!！?？,，；;]/.test(value)) score -= 3;
  if (/blog|博客|小窝|小屋|主页|站|site|log|space|空间|の/i.test(value)) score += 3;
  if (/愿|记录|分享|生活|技术|热爱|欢迎|一个|这里/.test(value)) score -= 1;
  if (value.length <= 24) score += 1;
  return score;
}

function descScore(value) {
  let score = 0;
  if (/[。.!！?？,，；;]/.test(value)) score += 2;
  if (/愿|记录|分享|生活|技术|热爱|欢迎|一个|这里/.test(value)) score += 2;
  score += Math.min(value.length / 20, 2);
  return score;
}

export function parseFriendQuickInput(rawText) {
  const lines = String(rawText || "")
    .split(/\r?\n/)
    .map(cleanLine)
    .filter(Boolean);
  const result = { name: "", desc: "", url: "", avatar: "" };
  const textLines = [];
  const urlLines = [];

  for (const line of lines) {
    const { label, value } = splitLabel(line);
    if (label && assignLabel(result, label, value)) continue;

    if (parseURL(value)) {
      urlLines.push(value);
    } else {
      textLines.push(value);
    }
  }

  for (const value of urlLines) {
    if (isAvatarURL(value) && !result.avatar) {
      result.avatar = value;
    } else if (!result.url) {
      result.url = value;
    } else if (!result.avatar) {
      result.avatar = value;
    }
  }

  const remainingTexts = textLines.filter((line) => line !== result.name && line !== result.desc);
  if (!result.name && remainingTexts.length) {
    remainingTexts.sort((a, b) => nameScore(b) - descScore(b) - (nameScore(a) - descScore(a)));
    result.name = remainingTexts.shift() || "";
  }
  if (!result.desc && remainingTexts.length) {
    remainingTexts.sort((a, b) => descScore(b) - descScore(a));
    result.desc = remainingTexts.shift() || "";
  }

  return result;
}
