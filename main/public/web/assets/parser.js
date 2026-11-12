/**
 * AI 智能识别 —— 把粘贴的文本解析成卡片
 * 用 Settings.ai() 读配置（默认接口 或 用户自定义）
 * 默认接口走限额检查；自定义接口不限
 */

import { Settings } from './settings.js';

// ---- 主入口 -----------------------------------------------------------------
export async function parseCards(text, opts = {}) {
  if (opts.useAI !== false) {
    const quota = Settings.checkQuota();
    const ai = Settings.ai();
    // 自定义接口直接试
    if (ai.source === 'custom' && ai.key) {
      try {
        const r = await callLLM(text, ai);
        return r;
      } catch (e) {
        console.warn('自定义 LLM 调用失败，回退到本地解析:', e);
      }
    }
    // 默认接口要看额度
    else if (ai.source === 'default') {
      if (quota.allow) {
        try {
          const r = await callLLM(text, ai);
          Settings.recordUse();   // 成功才计费
          return r;
        } catch (e) {
          console.warn('默认 LLM 调用失败，回退到本地解析:', e);
        }
      }
      // allow=false 或调用失败 → 走本地解析
    }
  }
  return parseLocal(text);
}

// ---- 本地启发式解析 ---------------------------------------------------------
export function parseLocal(text) {
  if (!text || !text.trim()) return { cards: [], warnings: ['输入为空'] };

  const blocks = splitBlocks(text);
  const cards = [];
  const warnings = [];

  blocks.forEach((block, i) => {
    const card = parseBlock(block, i);
    if (card) {
      cards.push(card);
      if (card._warn) warnings.push(`第 ${i + 1} 题：${card._warn}`);
    }
  });

  return { cards, warnings };
}

// 把整段文本切成"一题一块"
function splitBlocks(text) {
  text = text.replace(/\r\n/g, '\n').trim();

  // 按"行首是题号"切：1. / 1、 / 1) / (1) / ① / 一、 / 第1题 / Q1 / Q:
  const numberPattern = /(?:^|\n)\s*(?:(?:\d+|[①-⑳]|[一二三四五六七八九十]|[ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩ])\s*[.、)）]|\(\d+\)|第\s*\d+\s*[题問]|Q[:：\.\s]\s*\d*[:：]?)\s*/g;

  const matches = [];
  let m;
  while ((m = numberPattern.exec(text)) !== null) {
    matches.push(m.index + m[0].search(/\S/));
  }

  if (matches.length === 0) {
    // 没有题号，退化为按双换行切
    const parts = text.split(/\n\s*\n/).map(p => p.trim()).filter(Boolean);
    return parts.length ? parts : [text];
  }

  const blocks = [];
  for (let i = 0; i < matches.length; i++) {
    const start = matches[i];
    const end = i + 1 < matches.length ? matches[i + 1] : text.length;
    blocks.push(text.slice(start, end).trim());
  }
  return blocks;
}

// 解析单题
function parseBlock(block, idx) {
  // 1) 去掉开头题号
  const cleaned = block.replace(/^\s*(?:(?:\d+|[①-⑳]|[一二三四五六七八九十]|[ⅠⅡⅢⅣⅤⅥⅦⅧⅨⅩ])\s*[.、)）]|\(\d+\)|第\s*\d+\s*[题問]|Q[:：\.\s]\s*\d*[:：]?)\s*/, '').trim();

  // 2) 提取答案行（先找出来，避免被当成选项）
  let answer = '';
  let hint = '';
  let body = cleaned;

  // 答案：支持 答案/正确答案/Answer/Ans
  const ansMatch = body.match(/[\n\s]*(?:答案|正确答案|Answer|Ans|参考答案)\s*[:：=]\s*([A-Fa-f①-⑥1-6]+|[^\n]+?)(?=\n|$)/);
  if (ansMatch) {
    answer = ansMatch[1].trim();
    body = body.replace(ansMatch[0], '').trim();
  }

  // 解析：支持 解析/解释/Hint
  const hintMatch = body.match(/[\n\s]*(?:解析|解释|说明|Hint|Explanation)\s*[:：=]\s*([\s\S]+?)(?=\n\s*(?:答案|正确答案|$)|$)/);
  if (hintMatch) {
    hint = hintMatch[1].trim();
    body = body.replace(hintMatch[0], '').trim();
  }

  // 3) 找选项 —— 行首是 A. / A、 / A) / (A) / ① / *A) 等
  const optRe = /(?:^|\n)\s*(?:\(\s*([A-Fa-f])\s*\)|([A-Fa-f])\s*[.、)）]|([①-⑥]))\s*([^\n]+)/g;
  const opts = [];
  let om;
  while ((om = optRe.exec(body)) !== null) {
    const letter = om[1] || om[2] || circleToLetter(om[3]);
    const text = om[4].trim();
    if (text) opts.push({ letter: letter.toUpperCase(), text });
  }

  // 4) 题干 = body 减去所有选项行
  let prompt = body;
  if (opts.length) {
    const firstOptIdx = body.search(/(?:^|\n)\s*(?:\(\s*[A-Fa-f]\s*\)|[A-Fa-f]\s*[.、)）]|[①-⑥])/);
    if (firstOptIdx > 0) prompt = body.slice(0, firstOptIdx).trim();
  }
  // 去掉句末的提问引导
  prompt = prompt.replace(/^[\s\n]+|[\s\n]+$/g, '');

  if (!prompt) return null;

  // 5) 如果没有选项，按 Q&A 模式处理
  if (opts.length === 0) {
    // 看看下一段是不是答案
    const qaMatch = block.match(/^\s*(?:Q[:：]?|问[:：]?)?\s*(.+?)\n+\s*(?:A[:：]?|答[:：]?)\s*(.+)$/s);
    if (qaMatch) {
      return {
        _warn: '识别为问答题，已转为单选（需要补充干扰项）',
        prompt: qaMatch[1].trim(),
        options: [
          { text: qaMatch[2].trim(), correct: true },
          { text: '（请补充干扰项）', correct: false },
        ],
        hint,
      };
    }
    return { _warn: '未识别到选项，已自动跳过', prompt, options: [], hint, _skip: true };
  }

  // 6) 决定哪个是正确答案
  let correctLetter = '';
  if (answer) {
    // 答案可能是 "B" / "B、Environment" / "环境" / "②"
    const letterMatch = answer.match(/[A-Fa-f①-⑥]/);
    if (letterMatch) {
      correctLetter = (letterMatch[0].match(/[①-⑥]/) ? circleToLetter(letterMatch[0]) : letterMatch[0]).toUpperCase();
    } else {
      // 通过答案内容匹配选项
      const found = opts.find(o => o.text.includes(answer) || answer.includes(o.text));
      if (found) correctLetter = found.letter;
    }
  }
  // 仍未找到答案 —— 看选项里有没有 ★ ✓ 等标记
  if (!correctLetter) {
    const marked = opts.find(o => /[★✓✔√]|（正确）|\(正确\)|\*\*/.test(o.text));
    if (marked) {
      correctLetter = marked.letter;
      marked.text = marked.text.replace(/[★✓✔√*]|（正确）|\(正确\)/g, '').trim();
    }
  }

  const options = opts.map(o => ({
    text: o.text,
    correct: o.letter === correctLetter,
  }));

  // 没标答案 —— 默认 A 为正确，并警告
  let warn = '';
  if (!correctLetter) {
    options[0].correct = true;
    warn = '未检测到答案标记，已默认第一项为正确，请检查';
  }

  return { prompt, options, hint, _warn: warn };
}

function circleToLetter(ch) {
  const map = { '①':'A','②':'B','③':'C','④':'D','⑤':'E','⑥':'F' };
  return map[ch] || ch;
}

// ---- LLM 调用 -------------------------------------------------------
async function callLLM(text, ai) {
  const sys = `你是一个把用户粘贴的题目文本解析成结构化卡片的助手。返回严格的 JSON，格式：
{
  "cards": [
    { "prompt": "题干", "options": [{"text":"选项1","correct":false},{"text":"选项2","correct":true}], "hint": "可选解析" }
  ]
}
只返回 JSON，不要任何 markdown 包裹。每题至少 2 个选项，必须有一个 correct:true。`;

  const res = await fetch(ai.endpoint, {
    method: 'POST',
    headers: { 'Content-Type':'application/json', 'Authorization':'Bearer ' + ai.key },
    body: JSON.stringify({
      model: ai.model,
      messages: [
        { role:'system', content: sys },
        { role:'user', content: text },
      ],
      response_format: { type: 'json_object' },
      temperature: 0.2,
    }),
  });
  if (!res.ok) throw new Error('API ' + res.status);
  const data = await res.json();
  const content = data.choices[0].message.content;
  const obj = JSON.parse(content);
  return { cards: obj.cards || [], warnings: [], _from: 'llm' };
}
