(function () {
  var SKINS = [
    "moment-skin--aurora",
    "moment-skin--sakura",
    "moment-skin--ticket",
    "moment-skin--envelope",
    "moment-skin--watercolor",
    "moment-skin--starry",
    "moment-skin--candy",
    "moment-skin--journal",
    "moment-skin--wave",
    "moment-skin--frost",
    "moment-skin--sunset",
    "moment-skin--mint",
    "moment-skin--ribbon",
    "moment-skin--neon",
    "moment-skin--glass",
    "moment-skin--stamp",
  ];
  /* 归档 / 友链：保留美感，避免极端留白与异形边框 */
  var CALM_SKINS = [
    "moment-skin--aurora",
    "moment-skin--sakura",
    "moment-skin--watercolor",
    "moment-skin--starry",
    "moment-skin--candy",
    "moment-skin--frost",
    "moment-skin--sunset",
    "moment-skin--mint",
    "moment-skin--ribbon",
    "moment-skin--neon",
    "moment-skin--glass",
    "moment-skin--stamp",
  ];
  var POETIC = "moment-skin--mist";

  var FEEDS = [
    { root: ".moments-feed", tilt: true, skins: SKINS },
    { root: ".gallery-feed", tilt: true, skins: SKINS },
    { root: ".archive-timeline", tilt: false, skins: CALM_SKINS },
    { root: ".links-stack", tilt: false, skins: CALM_SKINS },
  ];

  function shuffle(arr) {
    for (var i = arr.length - 1; i > 0; i--) {
      var j = Math.floor(Math.random() * (i + 1));
      var t = arr[i];
      arr[i] = arr[j];
      arr[j] = t;
    }
    return arr;
  }

  function tilt() {
    return (Math.random() * 5 - 2.5).toFixed(2) + "deg";
  }

  function clearSkins(el) {
    SKINS.concat(POETIC).forEach(function (c) {
      el.classList.remove(c);
    });
  }

  function applyFeed(cfg) {
    var feed = document.querySelector(cfg.root);
    if (!feed) return;
    var cards = feed.querySelectorAll(".moment");
    if (!cards.length) return;

    var pool = shuffle(cfg.skins.slice());
    var pi = 0;

    cards.forEach(function (el) {
      clearSkins(el);
      el.style.setProperty("--moment-tilt", cfg.tilt ? tilt() : "0deg");

      if (el.classList.contains("moment--poetic")) {
        el.classList.add(POETIC);
        el.style.setProperty("--moment-tilt", "0deg");
        return;
      }

      el.classList.add(pool[pi % pool.length]);
      pi += 1;
    });
  }

  function apply() {
    FEEDS.forEach(applyFeed);
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", apply);
  } else {
    apply();
  }
})();
