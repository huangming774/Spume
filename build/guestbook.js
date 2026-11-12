(function (global) {
  var GB_KEY = "siteGuestbookMsgs";
  var D = "d" + "iv";

  function esc(s) {
    return String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function t(key, fallback) {
    return global.SiteI18n ? SiteI18n.t(key) : fallback;
  }

  function cardVariant(nick, text) {
    var n = (nick + text).length % 4;
    return "wl-item--v" + (n + 1);
  }

  // 糖果色系头像配色池（Akilar 风格）
  var avatarPalettes = [
    "linear-gradient(145deg, #ff9ec4, #c880c0)",
    "linear-gradient(145deg, #88d0c0, #68a8e0)",
    "linear-gradient(145deg, #f8a8c8, #8898e0)",
    "linear-gradient(145deg, #b898e8, #f098b8)",
    "linear-gradient(145deg, #98c8f0, #b098e8)",
    "linear-gradient(145deg, #f0b098, #e898c8)",
    "linear-gradient(145deg, #a8d8c8, #98b8e8)",
    "linear-gradient(145deg, #e8b8d8, #a8c0f0)"
  ];

  function avatarStyle(nick) {
    var idx = (nick || "桃").charCodeAt(0) % avatarPalettes.length;
    return avatarPalettes[idx];
  }

  function avatarChar(nick) {
    return (nick || "桃").charAt(0).toUpperCase();
  }

  // 浏览器 / 系统检测（Akilar 风格标签）
  function detectUA() {
    var ua = navigator.userAgent || "";
    var os = "";
    var browser = "";
    if (ua.indexOf("Windows") !== -1) os = "Windows";
    else if (ua.indexOf("Mac") !== -1) os = "Mac";
    else if (ua.indexOf("Linux") !== -1) os = "Linux";
    else if (ua.indexOf("Android") !== -1) os = "Android";
    else if (ua.indexOf("iPhone") !== -1 || ua.indexOf("iPad") !== -1) os = "iOS";
    else os = "未知";
    if (ua.indexOf("Edg") !== -1) browser = "Edge";
    else if (ua.indexOf("Chrome") !== -1) browser = "Chrome";
    else if (ua.indexOf("Firefox") !== -1) browser = "Firefox";
    else if (ua.indexOf("Safari") !== -1) browser = "Safari";
    else browser = "未知";
    return { os: os, browser: browser };
  }

  function formatDate(ts) {
    if (!ts) return "";
    var d = new Date(ts);
    var y = d.getFullYear();
    var m = String(d.getMonth() + 1).padStart(2, "0");
    var day = String(d.getDate()).padStart(2, "0");
    var h = String(d.getHours()).padStart(2, "0");
    var min = String(d.getMinutes()).padStart(2, "0");
    return y + "-" + m + "-" + day + " " + h + ":" + min;
  }

  function cardHtml(nick, text, ts, extraClass, uaInfo) {
    var variant = cardVariant(nick, text);
    var date = ts ? formatDate(ts) : "";
    var avatarGrad = avatarStyle(nick);
    var avaChar = avatarChar(nick);
    var dateHtml = date
      ? '<time class="wl-time" datetime="' + esc(date) + '">' + esc(date) + "</time>"
      : "";
    var ua = uaInfo || null;
    var tagHtml = ua
      ? '<span class="wl-tags"><span class="wl-tag">' +
        esc(ua.os) +
        '</span><span class="wl-tag">' +
        esc(ua.browser) +
        "</span></span>"
      : "";
    return (
      '<li class="wl-item ' +
      variant +
      (extraClass ? " " + extraClass : "") +
      '">' +
      '<span class="wl-avatar" aria-hidden="true" style="background:' +
      avatarGrad +
      '">' +
      esc(avaChar) +
      "</span>" +
      "<" +
      D +
      ' class="wl-main">' +
      '<header class="wl-header">' +
      '<span class="wl-name">' +
      esc(nick) +
      "</span>" +
      tagHtml +
      dateHtml +
      "</header>" +
      '<div class="wl-content"><p>' +
      esc(text) +
      "</p></div></" +
      D +
      "></li>"
    );
  }

  function readGB() {
    try {
      return JSON.parse(localStorage.getItem(GB_KEY) || "[]");
    } catch (e) {
      return [];
    }
  }

  function writeGB(arr) {
    localStorage.setItem(GB_KEY, JSON.stringify(arr));
  }

  function updateCount() {
    var root = document.getElementById("envList");
    var el = document.getElementById("msgLetterCount");
    if (root && el) {
      el.textContent = String(root.querySelectorAll(".wl-item").length);
    }
  }

  var _uaCache = null;
  function getUA() {
    if (!_uaCache) _uaCache = detectUA();
    return _uaCache;
  }

  function hydrate() {
    var root = document.getElementById("envList");
    if (!root || root.dataset.hydrated) return;
    root.dataset.hydrated = "1";
    if (!localStorage.getItem("gbCleaned")) { localStorage.removeItem(GB_KEY); localStorage.setItem("gbCleaned", "1"); }
			var list = readGB();
    var ua = getUA();
    // 最近一条（用户刚发的）带 UA 标签，旧数据不带
    var now = Date.now();
    list.forEach(function (m, i) {
      var wrap = document.createElement(D);
      var useUA = (now - (m.ts || 0) < 60000 && i === 0) ? ua : null;
      wrap.innerHTML = cardHtml(m.nick || t("msgAnonymous", "匿名"), m.text || "", m.ts, "", useUA);
      var node = wrap.firstElementChild;
      if (node) root.insertBefore(node, root.firstChild);
    });
    updateCount();
  }

  function addMsg() {
    var nameEl = document.getElementById("msgName");
    var textEl = document.getElementById("msgText");
    if (!nameEl || !textEl) return;
    var nick = nameEl.value.trim() || t("msgAnonymous", "匿名");
    var text = textEl.value.trim();
    if (!text) return;
    var ts = Date.now();
    var arr = readGB();
    arr.unshift({ nick: nick, text: text, ts: ts });
    writeGB(arr.slice(0, 80));
    var root = document.getElementById("envList");
    if (root) {
      var wrap = document.createElement(D);
      wrap.innerHTML = cardHtml(nick, text, ts, "wl-item--new", detectUA());
      var node = wrap.firstElementChild;
      if (node) {
        root.insertBefore(node, root.firstChild);
        node.addEventListener(
          "animationend",
          function () {
            node.classList.remove("wl-item--new");
          },
          { once: true }
        );
      }
    }
    nameEl.value = "";
    textEl.value = "";
    updateCount();
  }

  global.Guestbook = {
    GB_KEY: GB_KEY,
    esc: esc,
    cardHtml: cardHtml,
    readGB: readGB,
    writeGB: writeGB,
    hydrate: hydrate,
    addMsg: addMsg,
    updateCount: updateCount,
  };
  global.addMsg = addMsg;

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", hydrate);
  } else {
    hydrate();
  }
})(window);
