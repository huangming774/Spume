/**
 * 用户认证 + 数据隔离 —— 标准账号密码 + 邮箱验证
 * --------------------------------------------------------------
 * 登录：邮箱 + 密码
 * 注册：邮箱 + 密码 + 4 位邮箱验证码
 * 无后端版：密码哈希后存 localStorage，验证码本地生成并展示
 * 接真后端：把 register/login 改成 fetch 即可
 * --------------------------------------------------------------
 */

const USERS_KEY    = 'quizcard.users';     // [{id,email,nickname,avatar,passwordHash,createdAt}]
const SESSION_KEY  = 'quizcard.session';   // { userId, loginAt }
const CODES_KEY    = 'quizcard.codes';     // { email: { code, exp } }

function uid(p='u') {
  return p + '_' + Math.random().toString(36).slice(2,9) + Date.now().toString(36).slice(-4);
}
function load(k, def) {
  try { return JSON.parse(localStorage.getItem(k)) ?? def; } catch (e) { return def; }
}
function save(k, v) { localStorage.setItem(k, JSON.stringify(v)); }

// 简单哈希（前端版用 SubtleCrypto SHA-256，演示足够）
async function hashPassword(pw) {
  const buf = new TextEncoder().encode(pw + 'quizcard.salt.v1');
  const hash = await crypto.subtle.digest('SHA-256', buf);
  return [...new Uint8Array(hash)].map(b => b.toString(16).padStart(2,'0')).join('');
}

export const Auth = {
  currentUser() {
    const sess = load(SESSION_KEY, null);
    if (!sess) return null;
    const users = load(USERS_KEY, []);
    return users.find(u => u.id === sess.userId) || null;
  },
  isLoggedIn() {
    return !!this.currentUser();
  },

  // 邮箱是否已注册
  emailExists(email) {
    const e = (email || '').toLowerCase().trim();
    const users = load(USERS_KEY, []);
    return users.some(u => u.email === e);
  },

  // ===== 注册流程 =====
  // 1. 发送验证码
  async sendCode(email) {
    if (!isValidEmail(email)) throw new Error('邮箱格式不对');
    if (this.emailExists(email)) throw new Error('该邮箱已注册，请直接登录');
    const code = String(Math.floor(1000 + Math.random() * 9000));
    const codes = load(CODES_KEY, {});
    codes[email.toLowerCase()] = { code, exp: Date.now() + 5*60*1000 };
    save(CODES_KEY, codes);
    return { code, demo: true };       // 真后端把 code 删掉
  },

  // 2. 注册：邮箱 + 密码 + 验证码
  async register(email, password, code, nickname) {
    if (!isValidEmail(email))   throw new Error('邮箱格式不对');
    if (!password || password.length < 6) throw new Error('密码至少 6 位');
    if (password.length > 64)   throw new Error('密码太长');
    const e = email.toLowerCase().trim();
    if (this.emailExists(e))    throw new Error('该邮箱已注册，请直接登录');

    const codes = load(CODES_KEY, {});
    const rec = codes[e];
    if (!rec)                       throw new Error('请先发送验证码');
    if (Date.now() > rec.exp)       throw new Error('验证码已过期，请重新获取');
    if (String(code).trim() !== rec.code) throw new Error('验证码错误');

    delete codes[e]; save(CODES_KEY, codes);

    const users = load(USERS_KEY, []);
    const user = {
      id: uid('u'),
      email: e,
      nickname: (nickname || '').trim() || e.split('@')[0],
      avatar: makeAvatar(e),
      passwordHash: await hashPassword(password),
      createdAt: Date.now(),
    };
    users.push(user);
    save(USERS_KEY, users);
    save(SESSION_KEY, { userId: user.id, loginAt: Date.now() });
    return user;
  },

  // ===== 登录 =====
  async login(email, password) {
    if (!isValidEmail(email)) throw new Error('邮箱格式不对');
    if (!password)            throw new Error('请输入密码');
    const e = email.toLowerCase().trim();
    const users = load(USERS_KEY, []);
    const user = users.find(u => u.email === e);
    if (!user) throw new Error('该邮箱未注册');

    const hash = await hashPassword(password);
    if (hash !== user.passwordHash) throw new Error('密码错误');

    save(SESSION_KEY, { userId: user.id, loginAt: Date.now() });
    return user;
  },

  logout() {
    localStorage.removeItem(SESSION_KEY);
  },

  // 改昵称
  updateProfile(patch) {
    const u = this.currentUser();
    if (!u) throw new Error('未登录');
    const users = load(USERS_KEY, []);
    const target = users.find(x => x.id === u.id);
    Object.assign(target, patch);
    save(USERS_KEY, users);
    return target;
  },

  // 修改密码
  async changePassword(oldPw, newPw) {
    const u = this.currentUser();
    if (!u) throw new Error('未登录');
    if (!newPw || newPw.length < 6) throw new Error('新密码至少 6 位');
    const oldHash = await hashPassword(oldPw);
    if (oldHash !== u.passwordHash) throw new Error('原密码错误');
    const users = load(USERS_KEY, []);
    const target = users.find(x => x.id === u.id);
    target.passwordHash = await hashPassword(newPw);
    save(USERS_KEY, users);
  },

  // 数据按用户隔离
  dataKey() {
    const u = this.currentUser();
    return u ? 'quizcard.v1.' + u.id : 'quizcard.v1';
  },

  redirectToLogin() {
    const here = location.pathname.split('/').pop() || 'index.html';
    if (here === 'login.html') return;
    location.replace('login.html?next=' + encodeURIComponent(here + location.search));
  },
};

function isValidEmail(s) {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(s || '');
}
function makeAvatar(email) {
  const PALETTE = ['#576065','#665c60','#546259','#874d4d','#4d6087','#876d4d','#4d8767'];
  const hash = [...email].reduce((a,c) => a + c.charCodeAt(0), 0);
  return {
    letter: (email[0] || '?').toUpperCase(),
    color: PALETTE[hash % PALETTE.length],
  };
}
