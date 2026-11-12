/* ============ Showcase 交互：主题切换 + Lightbox ============ */
(function () {
  "use strict";

  // —— 主题切换 ——
  var THEME_KEY = "tg-theme";
  var root = document.documentElement;
  var media = window.matchMedia ? window.matchMedia("(prefers-color-scheme: dark)") : null;

  function applyTheme(mode) {
    if (mode === "dark") root.setAttribute("data-theme", "dark");
    else if (mode === "light") root.setAttribute("data-theme", "light");
    else {
      // auto / system
      if (media && media.matches) root.setAttribute("data-theme", "dark");
      else root.setAttribute("data-theme", "light");
    }
    // 更新按钮 aria-pressed
    var buttons = document.querySelectorAll(".tg-theme button[data-mode]");
    Array.prototype.forEach.call(buttons, function (b) {
      b.setAttribute("aria-pressed", b.dataset.mode === mode ? "true" : "false");
    });
  }

  function initTheme() {
    var saved;
    try { saved = localStorage.getItem(THEME_KEY); } catch (e) {}
    var mode = saved || "auto";
    applyTheme(mode);

    var buttons = document.querySelectorAll(".tg-theme button[data-mode]");
    Array.prototype.forEach.call(buttons, function (b) {
      b.addEventListener("click", function () {
        var m = b.dataset.mode;
        try { localStorage.setItem(THEME_KEY, m); } catch (e) {}
        applyTheme(m);
      });
    });

    if (media && media.addEventListener) {
      media.addEventListener("change", function () {
        var current;
        try { current = localStorage.getItem(THEME_KEY); } catch (e) {}
        if (!current || current === "auto") applyTheme("auto");
      });
    }
  }

  // —— Lightbox ——
  function initLightbox() {
    var overlay = document.getElementById("tg-lightbox");
    if (!overlay) return;
    var frame = overlay.querySelector("iframe");
    var title = overlay.querySelector(".tg-lightbox-title");
    var closeBtn = overlay.querySelector(".tg-lightbox-close");

    function open(src, label) {
      frame.src = src;
      title.textContent = label || "";
      overlay.classList.add("is-open");
      document.body.style.overflow = "hidden";
    }
    function close() {
      overlay.classList.remove("is-open");
      frame.src = "about:blank";
      document.body.style.overflow = "";
    }

    Array.prototype.forEach.call(
      document.querySelectorAll(".tg-mp-card[data-mp]"),
      function (card) {
        card.addEventListener("click", function () {
          open(card.dataset.mp, card.dataset.label || "");
        });
      }
    );
    closeBtn.addEventListener("click", close);
    overlay.addEventListener("click", function (e) {
      if (e.target === overlay) close();
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && overlay.classList.contains("is-open")) close();
    });
  }

  // —— iframe 加载失败时显示 fallback ——
  // 当 H5 应用未部署时 iframe 会加载失败或显示空白，覆盖 fallback 占位
  function initFrameFallback() {
    var frames = document.querySelectorAll("iframe[data-fallback]");
    Array.prototype.forEach.call(frames, function (f) {
      var fb = document.getElementById(f.dataset.fallback);
      if (!fb) return;
      var loaded = false;
      f.addEventListener("load", function () { loaded = true; fb.style.display = "none"; });
      // 4 秒还没 load → 显示 fallback（应对 X-Frame-Options 拒绝 / 404 等）
      setTimeout(function () {
        if (!loaded) fb.style.display = "grid";
      }, 4000);
    });
  }

  // —— 小程序卡片 iframe 缩放：把固定 375×812 缩到容器实际宽度 ——
  function initMpScale() {
    var MP_W = 375;
    var frames = document.querySelectorAll(".tg-mp-frame");
    if (!frames.length) return;

    function recalc() {
      Array.prototype.forEach.call(frames, function (f) {
        var w = f.clientWidth;
        if (!w) return;
        f.style.setProperty("--mp-scale", (w / MP_W).toFixed(4));
      });
    }

    recalc();
    // resize / 主题切换都重算
    window.addEventListener("resize", recalc);
    if (window.ResizeObserver) {
      var ro = new ResizeObserver(recalc);
      Array.prototype.forEach.call(frames, function (f) { ro.observe(f); });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () {
      initTheme();
      initLightbox();
      initFrameFallback();
      initMpScale();
    });
  } else {
    initTheme();
    initLightbox();
    initFrameFallback();
    initMpScale();
  }
})();
