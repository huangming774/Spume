/**
 * 全站晴/雨天切换 + Canvas 雨效
 */
(function () {
  'use strict';

  var STORAGE_KEY = 'siteWeather';
  var canvas, ctx, drops, animId, w, h;
  var running = false;

  function syncBodyClass() {
    if (!document.body) return;
    document.body.classList.toggle(
      'weather-rainy',
      document.documentElement.classList.contains('weather-rainy')
    );
  }

  function isRainy() {
    return document.documentElement.classList.contains('weather-rainy');
  }

  function labels() {
    var lang = window.SiteI18n && SiteI18n.currentLang ? SiteI18n.currentLang() : 'zh';
    if (lang === 'en') {
      return { toRain: 'Rainy', toSun: 'Sunny', iconRain: '☔', iconSun: '☀' };
    }
    return { toRain: '雨天', toSun: '晴天', iconRain: '☔', iconSun: '☀' };
  }

  function updateButton() {
    var btn = document.getElementById('weatherToggle');
    if (!btn) return;
    var L = labels();
    var rainy = isRainy();
    var icon = btn.querySelector('.weather-toggle-icon');
    var label = btn.querySelector('.weather-toggle-label');
    btn.setAttribute('aria-pressed', rainy ? 'true' : 'false');
    if (icon) icon.textContent = rainy ? L.iconSun : L.iconRain;
    if (label) label.textContent = rainy ? L.toSun : L.toRain;
    btn.title = rainy ? L.toSun : L.toRain;
  }

  function setWeather(rainy) {
    document.documentElement.classList.toggle('weather-rainy', rainy);
    syncBodyClass();
    try {
      localStorage.setItem(STORAGE_KEY, rainy ? 'rainy' : 'sunny');
    } catch (e) {}
    updateButton();
    if (rainy) startRain();
    else stopRain();
  }

  function createDrops(count) {
    var list = [];
    for (var i = 0; i < count; i++) {
      list.push({
        x: Math.random() * w,
        y: Math.random() * h,
        len: 8 + Math.random() * 22,
        speed: 14 + Math.random() * 18,
        opacity: 0.15 + Math.random() * 0.45,
        width: 0.6 + Math.random() * 1.4
      });
    }
    return list;
  }

  function resize() {
    if (!canvas) return;
    w = canvas.width = window.innerWidth;
    h = canvas.height = window.innerHeight;
    var density = Math.min(280, Math.floor((w * h) / 5500));
    drops = createDrops(density);
  }

  function draw() {
    if (!ctx || !running) return;
    ctx.clearRect(0, 0, w, h);

    var windX = 2.2;
    for (var i = 0; i < drops.length; i++) {
      var d = drops[i];
      var x2 = d.x + windX * (d.len / 8);
      var y2 = d.y + d.len;

      var grad = ctx.createLinearGradient(d.x, d.y, x2, y2);
      grad.addColorStop(0, 'rgba(160, 190, 220, 0)');
      grad.addColorStop(0.15, 'rgba(180, 210, 235, ' + d.opacity * 0.6 + ')');
      grad.addColorStop(0.5, 'rgba(210, 228, 245, ' + d.opacity + ')');
      grad.addColorStop(1, 'rgba(140, 175, 210, ' + d.opacity * 0.3 + ')');

      ctx.beginPath();
      ctx.strokeStyle = grad;
      ctx.lineWidth = d.width;
      ctx.lineCap = 'round';
      ctx.moveTo(d.x, d.y);
      ctx.lineTo(x2, y2);
      ctx.stroke();

      d.y += d.speed;
      d.x += windX;
      if (d.y > h + 30) {
        d.y = -d.len - Math.random() * 40;
        d.x = Math.random() * w;
      }
      if (d.x > w + 20) d.x = -20;
    }

    animId = requestAnimationFrame(draw);
  }

  function ensureCanvas() {
    canvas = document.getElementById('rainCanvas');
    if (!canvas) {
      canvas = document.createElement('canvas');
      canvas.id = 'rainCanvas';
      canvas.className = 'rain-canvas';
      canvas.setAttribute('aria-hidden', 'true');
      document.body.appendChild(canvas);
    }
    ctx = canvas.getContext('2d');
    resize();
  }

  function startRain() {
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    ensureCanvas();
    if (running) return;
    running = true;
    draw();
  }

  function stopRain() {
    running = false;
    if (animId) {
      cancelAnimationFrame(animId);
      animId = null;
    }
    if (ctx && canvas) ctx.clearRect(0, 0, canvas.width, canvas.height);
  }

  function init() {
    var btn = document.getElementById('weatherToggle');
    if (btn) {
      btn.addEventListener('click', function () {
        setWeather(!isRainy());
      });
    }

    window.addEventListener('site-i18n-applied', updateButton);
    window.addEventListener('resize', function () {
      if (running) resize();
    });

    syncBodyClass();
    updateButton();
    if (isRainy()) startRain();
  }

  window.SiteWeather = { setWeather: setWeather, isRainy: isRainy };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
