/**
 * 全局中英切换 — 各页引入本文件后，为元素加上 data-i18n / data-i18n-placeholder / data-i18n-title
 */
(function () {
  'use strict';

  var STR = {
    zh: {
      blogTitle: '桃之夭夭の创作屋',
      navHome: '首页',
      navArchive: '归档',
      navArticle: '文章',
      navGallery: '相册',
      navMsg: '留言墙',
      footerLine: '桃之夭夭 © 2026',
      dmPlaceholder: '发条弹幕…',
      dmSend: '发送',
      dmToggleOn: '关闭弹幕',
      dmToggleOff: '开启弹幕',
      dmStyleTitle: '切换弹幕艺术字',
      dmDemo1: '欢迎来到 桃之夭夭の创作屋 ✨',
      dmDemo2: 'Hello World! 🌸',
      msgFabTitle: '留言墙',
      msgModalTitle: '落笔星语',
      msgModalSub: '会保存在本机留言墙（与首页同步）',
      msgNick: '你的昵称',
      msgBody: '想说些什么…',
      msgSubmit: '送入星河',
      msgClose: '关闭',
      msgOpenFull: '打开留言墙页面',
      msgFrom: '来自',
      msgFromSuffix: ' 的留言',
      msgHudBrand: '桃之夭夭 · 星语终端',
      msgAnonymous: '匿名',
      msgWallHint: '把温柔与欢喜，留在桃夭的花园里',
      msgLocalNote: '留言暂存于本机浏览器，与首页同步',
      msgLettersTitle: '星语墙',
      msgLettersUnit: '条回响',
      archivesTitle: '归档',
      archivesLead: '按时间线浏览所有文章记录。',
      articlesTitle: '文章',
      metaWords: '字',
      metaReads: '阅读',
      metaMinutes: '约',
      metaMinSuffix: '分钟',
      galleryTitle: '相册',
      galleryAlbumAnime: '动漫',
      hexoComments: '评论',
      hexoNick: '昵称',
      hexoEmail: '邮箱',
      hexoUrl: '网址',
      galleryLead: '定格日常与热爱的一瞬。',
      galleryBack: '返回大类',
      galleryPickSub: '选择小类',
      galleryCatDaily: '日常生活',
      galleryCatArt: '手绘作品',
      galleryCatPhoto: '摄影随拍',
      galleryCatPet: '猫猫合集',
      galleryHintDaily: '烟火与日常',
      galleryHintArt: '纸上小宇宙',
      galleryHintPhoto: '光与静物',
      galleryHintPet: '毛茸茸治愈',
      gallerySubRecent: '最近',
      gallerySubOld: '往年',
      gallerySubSketches: '线稿',
      gallerySubColor: '上色',
      gallerySubStreet: '街拍',
      gallerySubStill: '静物',
      gallerySubNap: '打盹',
      gallerySubPlay: '玩耍',
      tagDone: '完成'
    },
    en: {
      blogTitle: 'Taozhiyaoyao Studio',
      navHome: 'Home',
      navArchive: 'Archives',
      navArticle: 'Posts',
      navGallery: 'Album',
      navMsg: 'Guestbook',
      footerLine: 'Taozhiyaoyao © 2026',
      dmPlaceholder: 'Send a danmaku…',
      dmSend: 'Send',
      dmToggleOn: 'Turn off danmaku',
      dmToggleOff: 'Turn on danmaku',
      dmStyleTitle: 'Cycle art font',
      dmDemo1: 'Welcome to Taozhiyaoyao Studio ✨',
      dmDemo2: 'Hello World! 🌸',
      msgFabTitle: 'Guestbook',
      msgModalTitle: 'Write a star message',
      msgModalSub: 'Saved locally and synced with the guestbook',
      msgNick: 'Your name',
      msgBody: 'What would you like to say…',
      msgSubmit: 'Send to the stars',
      msgClose: 'Close',
      msgOpenFull: 'Open guestbook page',
      msgFrom: 'From',
      msgFromSuffix: "'s message",
      msgHudBrand: 'Taozhi · Star Terminal',
      msgAnonymous: 'Anonymous',
      msgWallHint: 'Leave warmth and joy in the Taozhi garden',
      msgLocalNote: 'Messages are stored in this browser only and sync with the home page',
      msgLettersTitle: 'Starlit wall',
      msgLettersUnit: 'echoes',
      archivesTitle: 'Archives',
      archivesLead: 'Browse all posts on a simple timeline.',
      articlesTitle: 'Posts',
      metaWords: 'words',
      metaReads: 'reads',
      metaMinutes: '~',
      metaMinSuffix: 'min read',
      galleryTitle: 'Album',
      galleryAlbumAnime: 'Anime',
      hexoComments: 'Comments',
      hexoNick: 'Nickname',
      hexoEmail: 'Email',
      hexoUrl: 'Website',
      galleryLead: 'A snapshot of everyday life and what you love.',
      galleryBack: 'Back to categories',
      galleryPickSub: 'Pick a sub-album',
      galleryCatDaily: 'Daily life',
      galleryCatArt: 'Hand-drawn',
      galleryCatPhoto: 'Photography',
      galleryCatPet: 'Cats',
      galleryHintDaily: 'Everyday sparks',
      galleryHintArt: 'Paper universe',
      galleryHintPhoto: 'Light & still life',
      galleryHintPet: 'Fluffy therapy',
      gallerySubRecent: 'Recent',
      gallerySubOld: 'Older',
      gallerySubSketches: 'Sketches',
      gallerySubColor: 'Colored',
      gallerySubStreet: 'Street',
      gallerySubStill: 'Still life',
      gallerySubNap: 'Naps',
      gallerySubPlay: 'Play',
      tagDone: 'Done'
    }
  };

  function currentLang() {
    return localStorage.getItem('siteLang') === 'en' ? 'en' : 'zh';
  }

  function t(key) {
    var L = STR[currentLang()];
    return (L && L[key] !== undefined) ? L[key] : key;
  }

  function apply() {
    var lang = currentLang();
    var L = STR[lang];
    document.documentElement.setAttribute('lang', lang === 'en' ? 'en' : 'zh-CN');

    document.querySelectorAll('[data-i18n]').forEach(function (el) {
      var k = el.getAttribute('data-i18n');
      if (k && L[k] !== undefined) el.textContent = L[k];
    });
    document.querySelectorAll('[data-i18n-placeholder]').forEach(function (el) {
      var k = el.getAttribute('data-i18n-placeholder');
      if (k && L[k] !== undefined) el.placeholder = L[k];
    });
    document.querySelectorAll('[data-i18n-title]').forEach(function (el) {
      var k = el.getAttribute('data-i18n-title');
      if (k && L[k] !== undefined) el.title = L[k];
    });

    document.querySelectorAll('.i18n-zh').forEach(function (el) {
      el.style.display = lang === 'zh' ? '' : 'none';
    });
    document.querySelectorAll('.i18n-en').forEach(function (el) {
      el.style.display = lang === 'en' ? '' : 'none';
    });

    var btn = document.getElementById('siteLangToggle');
    if (btn) btn.textContent = lang === 'zh' ? 'English' : '中文';

    window.dispatchEvent(new CustomEvent('site-i18n-applied', { detail: { lang: lang } }));
  }

  function toggleLang() {
    localStorage.setItem('siteLang', currentLang() === 'zh' ? 'en' : 'zh');
    apply();
  }

  window.SiteI18n = {
    apply: apply,
    toggleLang: toggleLang,
    currentLang: currentLang,
    t: t
  };

  document.addEventListener('DOMContentLoaded', function () {
    apply();
    var b = document.getElementById('siteLangToggle');
    if (b) b.addEventListener('click', toggleLang);
  });
})();
