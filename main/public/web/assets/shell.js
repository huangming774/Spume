/**
 * 通用：注入顶栏 / 底栏 + 登录拦截 + 用户菜单
 *   mountShell('library')  默认会要求登录；传 { requireAuth: false } 跳过
 */
import { Auth } from './auth.js';
import { Theme } from './settings.js';

const NAV_ITEMS = [
  { key: 'library',  href: 'index.html',    icon: 'auto_stories', text: '卡片库' },
  { key: 'create',   href: 'create.html',   icon: 'add_circle',   text: '新建' },
  { key: 'history',  href: 'history.html',  icon: 'history',      text: '历史' },
];

function isShowcaseEmbed() {
  return new URLSearchParams(location.search).get('embed') === 'showcase';
}

export function mountShell(activeKey = 'library', opts = {}) {
  const requireAuth = opts.requireAuth !== false;
  const showcaseEmbed = isShowcaseEmbed();

  if (showcaseEmbed) {
    document.documentElement.classList.add('embed-showcase');
  }

  // 登录拦截
  if (requireAuth && !Auth.isLoggedIn()) {
    Auth.redirectToLogin();
    return;
  }

  const user = Auth.currentUser();

  // app bar
  const bar = document.createElement('header');
  bar.className = 'app-bar';
  bar.innerHTML = `
    <a class="icon-btn" href="index.html" aria-label="Home">
      <span class="material-symbols-outlined">style</span>
    </a>
    <div class="brand">AI QuizCard</div>
    <nav class="nav-desktop">
      ${NAV_ITEMS.map(it => `
        <a href="${it.href}" class="${activeKey===it.key?'active':''}">${it.text}</a>
      `).join('')}
    </nav>
    <div class="user-slot" style="display:flex;align-items:center;gap:4px">
      <button class="icon-btn" id="theme-toggle" title="主题">
        <span class="material-symbols-outlined" id="theme-ico">${themeIcon()}</span>
      </button>
      <a class="icon-btn" href="settings.html" title="设置">
        <span class="material-symbols-outlined">settings</span>
      </a>
      ${user ? userMenuHTML(user) : `<a class="icon-btn" href="login.html"><span class="material-symbols-outlined">login</span></a>`}
    </div>
  `;
  document.body.prepend(bar);

  // bottom nav (mobile)
  const nav = document.createElement('nav');
  nav.className = 'nav-mobile';
  nav.innerHTML = NAV_ITEMS.map(it => `
    <a href="${it.href}" class="${activeKey===it.key?'active':''}">
      <span class="material-symbols-outlined" style="${activeKey===it.key?"font-variation-settings:'FILL' 1;":''}">${it.icon}</span>
      <span>${it.text}</span>
    </a>
  `).join('');
  document.body.appendChild(nav);

  // 绑定用户菜单事件
  if (user) bindUserMenu();
  bindThemeToggle();
}

function themeIcon() {
  const t = Theme.get();
  if (t === 'dark')  return 'dark_mode';
  if (t === 'light') return 'light_mode';
  return 'contrast';   // auto
}
function bindThemeToggle() {
  const btn = document.getElementById('theme-toggle');
  if (!btn) return;
  btn.addEventListener('click', () => {
    // 浅 → 深 → 跟随 → 浅
    const cycle = { light: 'dark', dark: 'auto', auto: 'light' };
    const next = cycle[Theme.get()] || 'light';
    Theme.set(next);
    document.getElementById('theme-ico').textContent = themeIcon();
    toast(`已切换：${ {light:'☀️ 浅色',dark:'🌙 深色',auto:'⚙️ 跟随系统'}[next] }`);
  });
}

function userMenuHTML(u) {
  return `
    <div class="user-menu" id="user-menu">
      <button class="avatar" id="avatar-btn" title="${escapeAttr(u.nickname)}">
        ${avatarHTML(u)}
      </button>
      <div class="dropdown" id="user-dropdown">
        <div class="dd-head">
          <div class="dd-avatar">${avatarHTML(u, 40)}</div>
          <div class="dd-meta">
            <div class="dd-name">${escapeHtml(u.nickname)}</div>
            <div class="dd-email">${escapeHtml(u.email)}</div>
          </div>
        </div>
        <div class="dd-divider"></div>
        <button class="dd-item" id="dd-rename">
          <span class="material-symbols-outlined" style="font-size:16px">edit</span>修改昵称
        </button>
        <button class="dd-item danger" id="dd-logout">
          <span class="material-symbols-outlined" style="font-size:16px">logout</span>退出登录
        </button>
      </div>
    </div>
  `;
}

function avatarHTML(u, size) {
  const sz = size || 32;
  const a = u.avatar || { letter: (u.nickname||'?')[0].toUpperCase(), color: '#576065' };
  // 如果是真头像 URL（小程序登录有），直接用 img
  if (typeof a === 'string' && a.startsWith('http')) {
    return `<img src="${a}" style="width:${sz}px;height:${sz}px;border-radius:50%;object-fit:cover" />`;
  }
  const letter = (a.letter || '?').slice(0,1);
  const color  = a.color || '#576065';
  return `<span class="av-letter" style="width:${sz}px;height:${sz}px;background:${color};font-size:${Math.floor(sz*.45)}px">${letter}</span>`;
}

function bindUserMenu() {
  const btn = document.getElementById('avatar-btn');
  const dd  = document.getElementById('user-dropdown');
  btn.addEventListener('click', e => {
    e.stopPropagation();
    dd.classList.toggle('show');
  });
  document.addEventListener('click', () => dd.classList.remove('show'));

  document.getElementById('dd-logout').addEventListener('click', () => {
    if (!confirm('确定退出登录？')) return;
    Auth.logout();
    location.replace('login.html');
  });
  document.getElementById('dd-rename').addEventListener('click', () => {
    const u = Auth.currentUser();
    const n = prompt('新昵称：', u.nickname);
    if (n && n.trim() && n.trim() !== u.nickname) {
      Auth.updateProfile({ nickname: n.trim() });
      location.reload();
    }
  });
}

export function toast(msg, icon = 'info') {
  let el = document.querySelector('.toast');
  if (!el) {
    el = document.createElement('div');
    el.className = 'toast';
    document.body.appendChild(el);
  }
  el.innerHTML = `<span class="material-symbols-outlined" style="font-size:18px;">${icon}</span><span>${msg}</span>`;
  requestAnimationFrame(() => el.classList.add('show'));
  clearTimeout(el._t);
  el._t = setTimeout(() => el.classList.remove('show'), 1800);
}

export function qs(name) {
  return new URLSearchParams(location.search).get(name);
}

function escapeHtml(s='') { return String(s).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c])); }
function escapeAttr(s='') { return String(s).replace(/"/g,'&quot;'); }
