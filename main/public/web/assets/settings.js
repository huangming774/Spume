/**
 * 设置中心 + 主题切换 + AI 接口配置 + 用量统计
 * --------------------------------------------------------------
 * 默认接口：用 DEFAULT_AI 里的配置，每用户每天 5 次、累计 50 次（演示用）
 * 自定义接口：用户自己填，无限制
 * --------------------------------------------------------------
 * 真后端部署：把 DEFAULT_AI.endpoint/key 改成你自己的中转地址
 */

import { Auth } from './auth.js';

// 默认 AI（演示。生产请改成你自己的中转 endpoint，避免 key 暴露给客户端）
const DEFAULT_AI = {
  endpoint: 'https://api.openai.com/v1/chat/completions',
  model:    'gpt-4o-mini',
  // ⚠️ 演示用占位 key；真用请部署一个后端中转 + 鉴权
  key:      '',
  label:    '默认接口（受限）',
};

// 限额（每用户）
const QUOTA_DAILY = 5;
const QUOTA_TOTAL = 50;

// ===== storage helpers =====
function load(k, def) {
  try { return JSON.parse(localStorage.getItem(k)) ?? def; } catch (e) { return def; }
}
function save(k, v) { localStorage.setItem(k, JSON.stringify(v)); }
function userKey(suffix) {
  const u = Auth.currentUser?.();
  return 'quizcard.' + (u ? u.id : 'guest') + '.' + suffix;
}

// ===== 主题 =====
export const Theme = {
  // 'light' | 'dark' | 'auto'
  get()      { return load('quizcard.theme', 'auto'); },
  set(theme) {
    save('quizcard.theme', theme);
    this.apply();
  },
  // 实际生效的色调（解出 auto）
  resolved() {
    const t = this.get();
    if (t !== 'auto') return t;
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  },
  apply() {
    const t = this.resolved();
    document.documentElement.setAttribute('data-theme', t);
    document.documentElement.style.colorScheme = t;
  },
  // 监听系统主题变化（auto 模式）
  watchSystem() {
    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = () => { if (this.get() === 'auto') this.apply(); };
    if (mq.addEventListener) mq.addEventListener('change', handler);
    else mq.addListener(handler);   // Safari 兼容
  },
};
// 初始化（任何页面加载就立即应用，防止闪烁）
Theme.apply();
Theme.watchSystem();

// ===== AI 设置 =====
export const Settings = {
  // 返回完整 ai 配置 { source:'default'|'custom', endpoint, model, key, label }
  ai() {
    const cfg = load(userKey('ai'), null);
    if (cfg && cfg.source === 'custom' && cfg.key) {
      return { ...cfg };
    }
    // 默认
    return { source: 'default', ...DEFAULT_AI };
  },
  // 用户保存
  setAI(patch) {
    const cur = this.ai();
    const next = { ...cur, ...patch };
    save(userKey('ai'), next);
    return next;
  },
  // 切回默认
  useDefault() {
    save(userKey('ai'), { source: 'default' });
  },

  defaultsMeta() {
    return { ...DEFAULT_AI, quotaDaily: QUOTA_DAILY, quotaTotal: QUOTA_TOTAL };
  },

  // ===== 限额检查（仅 default 接口算）=====
  // 返回 { allow, reason?, used: {today, total} }
  checkQuota() {
    const ai = this.ai();
    if (ai.source === 'custom') {
      return { allow: true, used: this.usage(), unlimited: true };
    }
    // 默认接口必须配了 key 才能用
    if (!ai.key) {
      return { allow: false, reason: 'no-key', used: this.usage() };
    }
    const u = this.usage();
    if (u.today >= QUOTA_DAILY) {
      return { allow: false, reason: 'daily', used: u };
    }
    if (u.total >= QUOTA_TOTAL) {
      return { allow: false, reason: 'total', used: u };
    }
    return { allow: true, used: u };
  },

  // 记一次成功调用
  recordUse() {
    const today = todayKey();
    const obj = load(userKey('usage'), { total: 0, byDay: {} });
    obj.total += 1;
    obj.byDay[today] = (obj.byDay[today] || 0) + 1;
    save(userKey('usage'), obj);
    return this.usage();
  },

  usage() {
    const obj = load(userKey('usage'), { total: 0, byDay: {} });
    return {
      today: obj.byDay[todayKey()] || 0,
      total: obj.total,
      quotaDaily: QUOTA_DAILY,
      quotaTotal: QUOTA_TOTAL,
    };
  },
};

function todayKey() {
  const d = new Date();
  return d.getFullYear() + '-' + (d.getMonth()+1) + '-' + d.getDate();
}
