/**
 * AI QuizCard · 数据层（store.js）
 * --------------------------------------------------------------
 * 默认实现：localStorage（async 包装，方便日后切到后端）。
 * 切到真后端：
 *   1. 把 USE_REMOTE 改成 true
 *   2. 实现下面 remote.* 的 fetch 调用（已留好骨架）
 *   3. 其他业务代码（页面）一行不用改
 * --------------------------------------------------------------
 * 数据结构：
 *   Deck   { id, title, subject, desc, createdAt, lastPracticed?, mastery? }
 *   Card   { id, deckId, prompt, options:[{text,correct}], hint?, lang? }
 *   Session{ id, deckId, mode:'deep'|'quick', startedAt, finishedAt?,
 *            items:[{cardId, picked, correct, ms}] }
 */

const USE_REMOTE = false;
const REMOTE_BASE = ''; // e.g. 'https://api.example.com'

import { SEED } from './seed.js';
import { Auth } from './auth.js';

// 按登录用户拆 key —— 没登录就用旧 key（兼容游客）
function KEY() { return Auth.dataKey(); }

// ---- low-level storage ------------------------------------------------------
function load() {
  try {
    const raw = localStorage.getItem(KEY());
    if (raw) return JSON.parse(raw);
  } catch (e) { console.warn('store: load failed', e); }
  return null;
}
function save(state) {
  localStorage.setItem(KEY(), JSON.stringify(state));
}
function ensureSeed() {
  let s = load();
  if (!s || !s.decks || !s.decks.length) {
    s = JSON.parse(JSON.stringify(SEED));
    save(s);
  }
  if (!s.sessions) s.sessions = [];
  return s;
}

function uid(prefix = 'id') {
  return prefix + '_' + Math.random().toString(36).slice(2, 9) + Date.now().toString(36).slice(-4);
}

// ---- local impl -------------------------------------------------------------
const local = {
  async listDecks() {
    const s = ensureSeed();
    return s.decks.map(d => ({
      ...d,
      cardCount: s.cards.filter(c => c.deckId === d.id).length,
    }));
  },
  async getDeck(deckId) {
    const s = ensureSeed();
    const deck = s.decks.find(d => d.id === deckId);
    if (!deck) throw new Error('Deck not found: ' + deckId);
    const cards = s.cards.filter(c => c.deckId === deckId);
    return { ...deck, cards, cardCount: cards.length };
  },
  async createDeck({ title, subject, desc, scheduledAt }) {
    const s = ensureSeed();
    const deck = {
      id: uid('deck'),
      title, subject: subject || '通用', desc: desc || '',
      createdAt: Date.now(),
      scheduledAt: scheduledAt || null,
      mastery: 0,
    };
    s.decks.unshift(deck);
    save(s);
    return deck;
  },
  async updateDeck(deckId, patch) {
    const s = ensureSeed();
    const d = s.decks.find(x => x.id === deckId);
    if (!d) throw new Error('Deck not found');
    Object.assign(d, patch);
    save(s);
    return d;
  },
  async deleteDeck(deckId) {
    const s = ensureSeed();
    s.decks = s.decks.filter(d => d.id !== deckId);
    s.cards = s.cards.filter(c => c.deckId !== deckId);
    s.sessions = s.sessions.filter(x => x.deckId !== deckId);
    save(s);
  },
  async listCards(deckId) {
    const s = ensureSeed();
    return s.cards.filter(c => c.deckId === deckId);
  },
  async createCard(card) {
    const s = ensureSeed();
    const c = { id: uid('card'), ...card };
    s.cards.push(c);
    save(s);
    return c;
  },
  async updateCard(cardId, patch) {
    const s = ensureSeed();
    const c = s.cards.find(x => x.id === cardId);
    if (!c) throw new Error('Card not found');
    Object.assign(c, patch);
    save(s);
    return c;
  },
  async deleteCard(cardId) {
    const s = ensureSeed();
    s.cards = s.cards.filter(c => c.id !== cardId);
    save(s);
  },
  async saveSession(session) {
    const s = ensureSeed();
    const full = { id: uid('sess'), ...session };
    s.sessions.unshift(full);
    if (s.sessions.length > 100) s.sessions = s.sessions.slice(0, 100);
    // update deck stats
    const d = s.decks.find(x => x.id === session.deckId);
    if (d) {
      d.lastPracticed = session.finishedAt;
      const correct = session.items.filter(it => it.correct).length;
      d.mastery = Math.round((correct / session.items.length) * 100);
    }
    save(s);
    return full;
  },
  async getSession(sessionId) {
    const s = ensureSeed();
    return s.sessions.find(x => x.id === sessionId);
  },
  async listSessions() {
    const s = ensureSeed();
    return (s.sessions || []).slice().sort((a,b) => b.finishedAt - a.finishedAt);
  },
  async clearSessions() {
    const s = ensureSeed();
    s.sessions = [];
    save(s);
  },
  async resetAll() {
    localStorage.removeItem(KEY());
    ensureSeed();
  },
  async exportJSON() {
    return JSON.stringify(ensureSeed(), null, 2);
  },
  async importJSON(txt) {
    const obj = JSON.parse(txt);
    if (!obj.decks || !obj.cards) throw new Error('Invalid backup file');
    save(obj);
  },
};

// ---- remote skeleton (fill when backend is ready) --------------------------
const remote = {
  async listDecks()                 { return fetch(REMOTE_BASE + '/decks').then(r => r.json()); },
  async getDeck(id)                 { return fetch(REMOTE_BASE + '/decks/' + id).then(r => r.json()); },
  async createDeck(d)               { return fetch(REMOTE_BASE + '/decks', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(d) }).then(r => r.json()); },
  async updateDeck(id, patch)       { return fetch(REMOTE_BASE + '/decks/' + id, { method:'PATCH', headers:{'Content-Type':'application/json'}, body: JSON.stringify(patch) }).then(r => r.json()); },
  async deleteDeck(id)              { return fetch(REMOTE_BASE + '/decks/' + id, { method:'DELETE' }); },
  async listCards(deckId)           { return fetch(REMOTE_BASE + '/decks/' + deckId + '/cards').then(r => r.json()); },
  async createCard(c)               { return fetch(REMOTE_BASE + '/cards', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(c) }).then(r => r.json()); },
  async updateCard(id, patch)       { return fetch(REMOTE_BASE + '/cards/' + id, { method:'PATCH', headers:{'Content-Type':'application/json'}, body: JSON.stringify(patch) }).then(r => r.json()); },
  async deleteCard(id)              { return fetch(REMOTE_BASE + '/cards/' + id, { method:'DELETE' }); },
  async saveSession(s)              { return fetch(REMOTE_BASE + '/sessions', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(s) }).then(r => r.json()); },
  async getSession(id)              { return fetch(REMOTE_BASE + '/sessions/' + id).then(r => r.json()); },
  async resetAll()                  { throw new Error('Not supported on remote'); },
  async exportJSON()                { throw new Error('Use server tools'); },
  async importJSON()                { throw new Error('Use server tools'); },
};

export const Store = USE_REMOTE ? remote : local;

// ---- helpers used by UI -----------------------------------------------------
export function shuffle(arr) {
  const a = arr.slice();
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}
export function fmtDate(ts) {
  if (!ts) return '从未练习';
  const d = new Date(ts);
  const today = new Date(); today.setHours(0,0,0,0);
  const dd = new Date(d); dd.setHours(0,0,0,0);
  const diff = (today - dd) / 86400000;
  if (diff === 0) return '今天 ' + d.toTimeString().slice(0,5);
  if (diff === 1) return '昨天';
  if (diff < 7) return diff + ' 天前';
  return d.getMonth()+1 + '月' + d.getDate() + '日';
}
export function fmtDateTime(ts) {
  if (!ts) return '';
  const d = new Date(ts);
  return d.getFullYear() + '-' + String(d.getMonth()+1).padStart(2,'0') + '-' + String(d.getDate()).padStart(2,'0')
    + ' ' + String(d.getHours()).padStart(2,'0') + ':' + String(d.getMinutes()).padStart(2,'0');
}
