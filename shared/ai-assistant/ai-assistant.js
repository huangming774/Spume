(function () {
  "use strict";

  if (document.getElementById("blog-ai-root")) return;

  var API_CHAT = "/api/chat";
  var API_IMAGE = "/api/ai/image";
  var API_AUTH = "/api/auth";
  var OWNER_EMAIL = "173236231@qq.com";
  var LIMIT_MSG = "今日提问次数用完啦，明天再来问我吧～";
  var IMAGE_LIMIT_MSG = "今日生图次数用完啦，明天再来画吧～";
  var ERROR_MSG = "小精灵暂时走神了，请稍后再试～";
  var IMAGE_ERROR_MSG = "生图暂时失败了，请稍后再试～";
  var NOT_CONFIGURED_MSG =
    "小精灵还在沉睡中～站长配置 DeepSeek API Key 后就能聊天啦";
  var IMAGE_NOT_CONFIGURED_MSG =
    "生图还没配置 Agnes API Key，站长配好后就能用了。";

  var LOGO_URL =
    "/cos/AI%E8%87%AA%E5%8A%A8%E5%8C%96%E5%8D%9A%E5%AE%A2%E5%9B%BE%E7%89%87/main/img/logo.png";
  var LIMITS = { chat: 2000, image: 1200 };
  var API_AVATAR = "/api/auth/avatar";

  var state = {
    open: false,
    mode: "chat",
    authMode: "login",
    authModalOpen: false,
    imageModalOpen: false,
    sending: false,
    chatEnabled: true,
    imageEnabled: false,
    limit: 10,
    used: 0,
    remaining: 10,
    imageLimit: 0,
    imageUsed: 0,
    imageRemaining: 0,
    isLogin: false,
    unlimited: false,
    user: null,
    challengeToken: "",
    securityQuestion: "",
  };

  var ui = {};

  function el(tag, cls, text) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    if (text != null) node.textContent = text;
    return node;
  }

  function userAvatarURL() {
    return (state.user && state.user.avatar) || "";
  }

  function userInitial() {
    var name = userDisplayName(state.user);
    return (name || "游").slice(0, 1);
  }

  // 填充一个头像容器：有 avatar 用图片，否则用首字
  function fillAvatar(container, opts) {
    container.innerHTML = "";
    var url = userAvatarURL();
    if (url) {
      var img = el("img");
      img.src = url;
      img.alt = "头像";
      container.appendChild(img);
    } else {
      container.textContent = userInitial();
    }
    if (opts && opts.clickable) {
      container.classList.add("is-clickable");
      container.title = "点击更换头像";
      container.addEventListener("click", pickAvatar);
    }
  }

  function pickAvatar() {
    if (!state.isLogin) {
      openAuthModal("login");
      return;
    }
    var input = document.createElement("input");
    input.type = "file";
    input.accept = "image/png,image/jpeg,image/webp,image/gif";
    input.addEventListener("change", function () {
      var file = input.files && input.files[0];
      if (!file) return;
      uploadAvatar(file);
    });
    input.click();
  }

  function uploadAvatar(file) {
    if (!ui.avatarBtn) return;
    var form = new FormData();
    form.append("file", file);
    ui.avatarBtn.disabled = true;
    var prev = ui.avatarBtn.innerHTML;
    ui.avatarBtn.innerHTML = "";
    ui.avatarBtn.appendChild(el("span", "bai-avatar-uploading", "上传中"));
    authFetchRaw(API_AVATAR, { method: "POST", body: form }).then(function (res) {
      return res.json().then(function (d) {
        return { ok: res.ok, status: res.status, data: d };
      });
    }).then(function (res) {
      if (res.ok && res.data && res.data.user) {
        state.user = res.data.user;
        updateAuthBar();
        appendMsg(ui.messages, "system", "头像更新好啦～");
      } else {
        appendMsg(ui.messages, "system", (res.data && res.data.message) || "头像上传失败，请稍后再试。");
      }
    }).catch(function () {
      appendMsg(ui.messages, "system", "头像上传失败，请稍后再试。");
    }).finally(function () {
      ui.avatarBtn.disabled = false;
      updateAuthBar();
    });
  }

  function authFetchRaw(path, options) {
    return fetch(API_AUTH + path, Object.assign(
      { credentials: "include" },
      options || {}
    ));
  }

  function authFetch(path, options) {
    return fetch(API_AUTH + path, Object.assign(
      { credentials: "include", headers: { Accept: "application/json" } },
      options || {}
    )).then(function (r) {
      return r.json().then(function (d) {
        return { ok: r.ok, status: r.status, data: d };
      });
    });
  }

  function getPageContext() {
    if (
      window.BlogAIPageContext &&
      typeof window.BlogAIPageContext.collectPageContext === "function"
    ) {
      return window.BlogAIPageContext.collectPageContext(document, location);
    }
    return {
      pageUrl: location.href,
      pageTitle: (document.title || "").trim(),
      pagePath: location.pathname || "/",
      headings: [],
      visibleText: "",
    };
  }

  function updatePageHint() {
    if (!ui.pageHint) return;
    var ctx = getPageContext();
    var short =
      ctx.pageTitle.length > 36
        ? ctx.pageTitle.slice(0, 36) + "…"
        : ctx.pageTitle;
    ui.pageHint.textContent = "当前页面：" + (short || ctx.pagePath);
    ui.pageHint.title = ctx.pageUrl;
  }

  function fallbackNameFromEmail(email) {
    var value = (email || "").trim();
    var at = value.indexOf("@");
    var local = at > 0 ? value.slice(0, at) : value;
    return (local || "已登录").slice(0, 12);
  }

  function userDisplayName(user) {
    var displayName = ((user && user.displayName) || "").trim();
    return displayName || fallbackNameFromEmail((user && user.email) || "");
  }

  function updateAuthBar() {
    if (!ui.authBar) return;
    ui.authBar.innerHTML = "";
    if (state.user) {
      ui.avatarBtn = el("button", "bai-avatar");
      ui.avatarBtn.type = "button";
      ui.avatarBtn.setAttribute("aria-label", "更换头像");
      fillAvatar(ui.avatarBtn, { clickable: true });
      var who = el("span", "blog-ai-auth-user", userDisplayName(state.user));
      if (state.user.email) who.title = state.user.email;
      var rename = el("button", "blog-ai-auth-link", "改名");
      rename.type = "button";
      rename.addEventListener("click", function () {
        openAuthModal("profile");
      });
      var out = el("button", "blog-ai-auth-link", "退出");
      out.type = "button";
      out.addEventListener("click", onLogout);
      var actions = el("span", "blog-ai-auth-actions");
      actions.appendChild(rename);
      actions.appendChild(out);
      ui.authBar.appendChild(ui.avatarBtn);
      ui.authBar.appendChild(who);
      ui.authBar.appendChild(actions);
      return;
    }
    ui.avatarBtn = el("button", "bai-avatar");
    ui.avatarBtn.type = "button";
    ui.avatarBtn.setAttribute("aria-label", "登录后可更换头像");
    fillAvatar(ui.avatarBtn, { clickable: true });
    ui.authBar.appendChild(ui.avatarBtn);
    var login = el("button", "blog-ai-auth-link", "登录 / 注册");
    login.type = "button";
    login.addEventListener("click", function () {
      openAuthModal("login");
    });
    ui.authBar.appendChild(login);
  }

  function ensureAuthModal() {
    if (ui.authOverlay) return;

    ui.authOverlay = el("div");
    ui.authOverlay.id = "acct-auth-overlay";
    ui.authOverlay.className = "acct-auth-overlay";
    ui.authOverlay.setAttribute("aria-hidden", "true");
    ui.authOverlay.addEventListener("click", function (e) {
      if (e.target === ui.authOverlay) closeAuthModal();
    });

    ui.authModal = el("div");
    ui.authModal.className = "acct-auth-modal";
    ui.authModal.setAttribute("role", "dialog");
    ui.authModal.setAttribute("aria-modal", "true");
    ui.authModal.setAttribute("aria-labelledby", "acct-auth-title");
    ui.authModal.addEventListener("click", function (e) {
      e.stopPropagation();
    });

    var closeBtn = el("button", "acct-auth-close", "×");
    closeBtn.type = "button";
    closeBtn.setAttribute("aria-label", "关闭");
    closeBtn.addEventListener("click", closeAuthModal);

    var brand = el("div", "acct-auth-brand");
    var avatar = el("div", "acct-auth-avatar", "✦");
    brand.appendChild(avatar);
    ui.authTitle = el("h2", "acct-auth-title");
    ui.authTitle.id = "acct-auth-title";
    brand.appendChild(ui.authTitle);
    ui.authSubtitle = el("p", "acct-auth-subtitle");
    brand.appendChild(ui.authSubtitle);

    ui.authTabs = el("div", "acct-auth-tabs");
    ui.authModalBody = el("div", "acct-auth-body");

    ui.authModal.appendChild(closeBtn);
    ui.authModal.appendChild(brand);
    ui.authModal.appendChild(ui.authTabs);
    ui.authModal.appendChild(ui.authModalBody);
    ui.authOverlay.appendChild(ui.authModal);
    document.body.appendChild(ui.authOverlay);

    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && state.authModalOpen) closeAuthModal();
      if (e.key === "Escape" && state.imageModalOpen) closeImageModal();
    });
  }

  function openAuthModal(mode) {
    ensureAuthModal();
    state.authMode = mode || "login";
    state.authModalOpen = true;
    ui.authOverlay.classList.add("is-visible");
    ui.authOverlay.setAttribute("aria-hidden", "false");
    document.body.classList.add("acct-auth-open");
    renderAuthModalContent();
  }

  function closeAuthModal() {
    if (!ui.authOverlay) return;
    state.authModalOpen = false;
    ui.authOverlay.classList.remove("is-visible");
    ui.authOverlay.setAttribute("aria-hidden", "true");
    document.body.classList.remove("acct-auth-open");
  }

  function setAuthError(msg) {
    if (!ui.authError) return;
    ui.authError.textContent = msg || "";
    ui.authError.hidden = !msg;
  }

  function renderAuthTabs() {
    if (!ui.authTabs) return;
    ui.authTabs.innerHTML = "";
    if (state.authMode === "security" || state.authMode === "profile") {
      ui.authTabs.hidden = true;
      return;
    }
    ui.authTabs.hidden = false;

    var loginTab = el("button", "acct-auth-tab", "登录");
    loginTab.type = "button";
    var regTab = el("button", "acct-auth-tab", "注册");
    regTab.type = "button";

    if (state.authMode === "login") loginTab.classList.add("is-active");
    else regTab.classList.add("is-active");

    loginTab.addEventListener("click", function () {
      state.authMode = "login";
      renderAuthModalContent();
    });
    regTab.addEventListener("click", function () {
      state.authMode = "register";
      renderAuthModalContent();
    });

    ui.authTabs.appendChild(loginTab);
    ui.authTabs.appendChild(regTab);
  }

  function renderAuthModalContent() {
    if (!ui.authModalBody) return;
    ui.authModalBody.innerHTML = "";
    setAuthError("");
    renderAuthTabs();

    if (state.authMode === "profile") {
      ui.authTitle.textContent = "修改昵称";
      ui.authSubtitle.textContent = "这个名字会显示在 AI 小窗，也会用于主页留言板";
      ui.authModalBody.appendChild(el("label", "acct-auth-label", "昵称"));
      var name = document.createElement("input");
      name.type = "text";
      name.className = "acct-auth-input";
      name.id = "blog-ai-auth-display-name";
      name.placeholder = "最多 12 个字";
      name.maxLength = 12;
      name.value = userDisplayName(state.user);
      name.autocomplete = "nickname";
      ui.authModalBody.appendChild(name);
      ui.authModalBody.appendChild(
        el("p", "acct-auth-hint", "保存后，新发表的留言会使用这个名字；历史留言保持原样。")
      );
      ui.authError = el("p", "acct-auth-error");
      ui.authError.hidden = true;
      ui.authModalBody.appendChild(ui.authError);
      var save = el("button", "acct-auth-primary", "保存昵称");
      save.type = "button";
      save.addEventListener("click", onUpdateProfile);
      ui.authModalBody.appendChild(save);
      var cancel = el("button", "acct-auth-text-btn", "先不改了");
      cancel.type = "button";
      cancel.addEventListener("click", closeAuthModal);
      ui.authModalBody.appendChild(cancel);
      name.focus();
      name.select();
      return;
    }

    if (state.authMode === "security") {
      ui.authTitle.textContent = "安全验证";
      ui.authSubtitle.textContent = "站长账号需完成验证后启用无限 AI 额度";
      ui.authModalBody.appendChild(
        el("label", "acct-auth-label", state.securityQuestion || "你现在的学号")
      );
      var ans = document.createElement("input");
      ans.type = "text";
      ans.className = "acct-auth-input";
      ans.id = "blog-ai-auth-answer";
      ans.placeholder = "请输入答案";
      ans.autocomplete = "off";
      ui.authModalBody.appendChild(ans);
      ui.authError = el("p", "acct-auth-error");
      ui.authError.hidden = true;
      ui.authModalBody.appendChild(ui.authError);
      var btnOk = el("button", "acct-auth-primary", "确认验证");
      btnOk.type = "button";
      btnOk.addEventListener("click", onVerifySecurity);
      ui.authModalBody.appendChild(btnOk);
      var back = el("button", "acct-auth-text-btn", "返回重新登录");
      back.type = "button";
      back.addEventListener("click", function () {
        openAuthModal("login");
      });
      ui.authModalBody.appendChild(back);
      ans.focus();
      return;
    }

    var isLogin = state.authMode === "login";
    ui.authTitle.textContent = isLogin ? "账号登录" : "注册账号";
    ui.authSubtitle.textContent = isLogin
      ? "登录后每日可提问 50 次（游客 10 次）"
      : "注册成功将自动登录，无需邮箱验证";

    ui.authModalBody.appendChild(el("label", "acct-auth-label", "邮箱"));
    var email = document.createElement("input");
    email.type = "email";
    email.className = "acct-auth-input";
    email.id = "blog-ai-auth-email";
    email.placeholder = "请输入邮箱";
    email.autocomplete = "username";
    ui.authModalBody.appendChild(email);

    if (!isLogin) {
      ui.authModalBody.appendChild(
        el("p", "acct-auth-hint", OWNER_EMAIL + " 为站长专用邮箱，不可注册")
      );
    }

    ui.authModalBody.appendChild(el("label", "acct-auth-label", "密码"));
    var pw = document.createElement("input");
    pw.type = "password";
    pw.className = "acct-auth-input";
    pw.id = "blog-ai-auth-password";
    pw.placeholder = "至少 8 位字符";
    pw.minLength = 8;
    pw.autocomplete = isLogin ? "current-password" : "new-password";
    ui.authModalBody.appendChild(pw);

    ui.authError = el("p", "acct-auth-error");
    ui.authError.hidden = true;
    ui.authModalBody.appendChild(ui.authError);

    var submit = el("button", "acct-auth-primary", isLogin ? "登 录" : "注册并登录");
    submit.type = "button";
    submit.addEventListener("click", isLogin ? onLogin : onRegister);
    ui.authModalBody.appendChild(submit);

    var foot = el("p", "acct-auth-foot");
    foot.textContent = "登录用于 AI 提问额度，与博客小精灵聊天入口分离";
    ui.authModalBody.appendChild(foot);

    email.focus();
  }

  function finishAuthSuccess(message) {
    closeAuthModal();
    updateAuthBar();
    refreshQuota();
    window.dispatchEvent(new CustomEvent("blog-auth-state-changed", {
      detail: { user: state.user, isLogin: state.isLogin, unlimited: state.unlimited },
    }));
    if (message && ui.messages) appendMsg(ui.messages, "bot", message);
  }

  function refreshAuthMe() {
    return authFetch("/me", { method: "GET" }).then(function (res) {
      if (res.ok && res.data && res.data.loggedIn && res.data.user) {
        state.user = res.data.user;
        state.isLogin = true;
        state.unlimited = !!res.data.unlimited;
      } else {
        state.user = null;
        state.isLogin = false;
        state.unlimited = false;
      }
      updateAuthBar();
      updateQuotaLabel();
    });
  }

  function onRegister() {
    var email = (document.getElementById("blog-ai-auth-email") || {}).value || "";
    var password = (document.getElementById("blog-ai-auth-password") || {}).value || "";
    setAuthError("");
    authFetch("/register", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email: email.trim(), password: password }),
    }).then(function (res) {
      if (!res.ok) {
        setAuthError((res.data && res.data.message) || "注册失败");
        return;
      }
      state.user = res.data.user;
      state.isLogin = true;
      state.unlimited = false;
      finishAuthSuccess("注册成功～现在每天可以问我 50 次啦！");
    });
  }

  function onLogin() {
    var email = (document.getElementById("blog-ai-auth-email") || {}).value || "";
    var password = (document.getElementById("blog-ai-auth-password") || {}).value || "";
    setAuthError("");
    authFetch("/login", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ email: email.trim(), password: password }),
    }).then(function (res) {
      if (!res.ok) {
        setAuthError((res.data && res.data.message) || "登录失败");
        return;
      }
      if (res.data.needsSecurityQuestion) {
        state.challengeToken = res.data.challengeToken || "";
        state.securityQuestion = res.data.securityQuestion || "你现在的学号";
        state.user = res.data.user;
        state.authMode = "security";
        renderAuthModalContent();
        return;
      }
      state.user = res.data.user;
      state.isLogin = true;
      state.unlimited = !!res.data.unlimited;
      finishAuthSuccess("登录成功～");
    });
  }

  function onVerifySecurity() {
    var answer = (document.getElementById("blog-ai-auth-answer") || {}).value || "";
    setAuthError("");
    authFetch("/verify-security", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        challengeToken: state.challengeToken,
        answer: answer.trim(),
      }),
    }).then(function (res) {
      if (!res.ok) {
        setAuthError((res.data && res.data.message) || "验证失败");
        return;
      }
      state.user = res.data.user;
      state.isLogin = true;
      state.unlimited = true;
      finishAuthSuccess("验证通过～站长无限额度已启用 ✦");
    });
  }

  function onUpdateProfile() {
    var displayName = (document.getElementById("blog-ai-auth-display-name") || {}).value || "";
    setAuthError("");
    authFetch("/profile", {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ displayName: displayName.trim() }),
    }).then(function (res) {
      if (!res.ok) {
        setAuthError((res.data && res.data.message) || "昵称更新失败");
        return;
      }
      state.user = res.data.user || Object.assign({}, state.user, {
        displayName: displayName.trim(),
      });
      state.isLogin = true;
      window.dispatchEvent(new CustomEvent("blog-auth-profile-updated", {
        detail: { user: state.user },
      }));
      finishAuthSuccess("昵称更新好啦～之后留言会用这个名字。");
    });
  }

  function onLogout() {
    authFetch("/logout", { method: "POST" }).then(function () {
      state.user = null;
      state.isLogin = false;
      state.unlimited = false;
      updateAuthBar();
      window.dispatchEvent(new CustomEvent("blog-auth-state-changed", {
        detail: { user: null, isLogin: false, unlimited: false },
      }));
      refreshAllQuotas();
      appendMsg(ui.messages, "system", "已退出登录");
    });
  }

  function setMode(mode) {
    state.mode = mode === "image" ? "image" : "chat";
    if (ui.modeChat) ui.modeChat.classList.toggle("is-active", state.mode === "chat");
    if (ui.modeImage) ui.modeImage.classList.toggle("is-active", state.mode === "image");
    var limit = LIMITS[state.mode] || LIMITS.chat;
    if (ui.input) {
      ui.input.placeholder =
        state.mode === "image" ? "描述想生成的画面…" : "输入你的问题…";
      ui.input.setAttribute("maxlength", String(limit));
    }
    if (ui.countMax) ui.countMax.textContent = String(limit);
    if (ui.send) {
      ui.send.textContent = state.mode === "image" ? "生成" : "发送";
    }
    updateCount();
    updateQuotaLabel();
    if (state.mode === "image") refreshImageQuota();
  }

  function updateCount() {
    if (!ui.count || !ui.input) return;
    ui.count.textContent = String((ui.input.value || "").length);
  }

  function mount() {
    var root = el("div");
    root.id = "blog-ai-root";

    var toggle = el("button", null, "✦ 问问博客");
    toggle.id = "blog-ai-toggle";
    toggle.type = "button";
    toggle.setAttribute("aria-expanded", "false");
    toggle.setAttribute("aria-controls", "bai-overlay");

    // —— 中央放大窗口 ——
    ui.overlay = el("div");
    ui.overlay.id = "bai-overlay";
    ui.overlay.className = "bai-overlay";
    ui.overlay.setAttribute("aria-hidden", "true");

    var backdrop = el("button", "bai-backdrop");
    backdrop.type = "button";
    backdrop.setAttribute("aria-label", "关闭");
    backdrop.addEventListener("click", function () { setOpen(false); });

    var app = el("div");
    app.className = "bai-app";
    app.id = "blog-ai-panel";
    app.setAttribute("role", "dialog");
    app.setAttribute("aria-modal", "true");
    app.setAttribute("aria-label", "博客小精灵");

    // —— 左侧栏 ——
    var aside = el("aside", "bai-aside");

    var brand = el("div", "bai-brand");
    var brandMark = el("div", "bai-brand-mark");
    var logo = el("img");
    logo.src = LOGO_URL;
    logo.alt = "博客小精灵";
    brandMark.appendChild(logo);
    var brandText = el("div", "bai-brand-text");
    brandText.appendChild(el("b", null, "博客小精灵"));
    brandText.appendChild(el("span", null, "taozhiyy · AI"));
    brand.appendChild(brandMark);
    brand.appendChild(brandText);
    aside.appendChild(brand);

    ui.newChat = el("button", "bai-new-chat", "＋ 新对话");
    ui.newChat.type = "button";
    ui.newChat.addEventListener("click", function () {
      if (!ui.messages) return;
      ui.messages.innerHTML = "";
      appendMsg(ui.messages, "bot", "新对话已开始～想聊点什么？");
    });
    aside.appendChild(ui.newChat);

    aside.appendChild(el("div", "bai-side-label", "模式"));
    ui.modeTabs = el("div", "blog-ai-mode-tabs");
    ui.modeChat = el("button", "blog-ai-mode-tab is-active", "聊天");
    ui.modeChat.type = "button";
    ui.modeImage = el("button", "blog-ai-mode-tab", "生图");
    ui.modeImage.type = "button";
    ui.modeTabs.appendChild(ui.modeChat);
    ui.modeTabs.appendChild(ui.modeImage);
    aside.appendChild(ui.modeTabs);

    aside.appendChild(el("div", "bai-side-label", "当前页面"));
    var ctxCard = el("div", "bai-context-card");
    ctxCard.appendChild(el("b", null, "Page Context"));
    ui.pageHint = el("p");
    ui.pageHint.id = "blog-ai-page-hint";
    ui.pageHint.className = "blog-ai-page-hint";
    ctxCard.appendChild(ui.pageHint);
    aside.appendChild(ctxCard);

    var sideFoot = el("div", "bai-side-foot");
    ui.quota = el("span");
    ui.quota.id = "blog-ai-quota";
    ui.quota.className = "bai-quota";
    ui.quota.textContent = "今日剩余：—/—";
    sideFoot.appendChild(ui.quota);
    ui.authBar = el("div");
    ui.authBar.id = "blog-ai-auth-bar";
    ui.authBar.className = "blog-ai-auth-bar";
    sideFoot.appendChild(ui.authBar);
    aside.appendChild(sideFoot);
    updatePageHint();

    // —— 主区 ——
    var main = el("div", "bai-main");

    var topbar = el("div", "bai-topbar");
    var topTitle = el("div");
    topTitle.appendChild(el("h2", null, "✦ 博客小精灵"));
    ui.subtitle = el("p", null, "可以问我当前页面相关问题，也可以生成一张小图");
    topTitle.appendChild(ui.subtitle);
    topbar.appendChild(topTitle);
    var spacer = el("div", "bai-topbar-spacer");
    topbar.appendChild(spacer);
    var closeBtn = el("button", "bai-win-btn bai-win-close", "✕");
    closeBtn.type = "button";
    closeBtn.setAttribute("aria-label", "关闭");
    closeBtn.title = "关闭";
    closeBtn.addEventListener("click", function () { setOpen(false); });
    topbar.appendChild(closeBtn);
    main.appendChild(topbar);

    var body = el("div");
    body.id = "blog-ai-body";
    body.className = "bai-body";

    ui.messages = el("div");
    ui.messages.id = "blog-ai-messages";
    appendMsg(ui.messages, "bot", "你好呀～点左侧「登录 / 注册」可提升到 50 次/天～");

    body.appendChild(ui.messages);
    main.appendChild(body);

    ui.chatForm = el("form");
    ui.chatForm.id = "blog-ai-form";
    ui.chatForm.className = "bai-composer";
    var shell = el("div", "bai-composer-shell");
    var input = document.createElement("textarea");
    input.id = "blog-ai-input";
    ui.input = input;
    input.rows = 1;
    input.placeholder = "输入你的问题…";
    input.setAttribute("maxlength", String(LIMITS.chat));
    var send = el("button", null, "发送");
    ui.send = send;
    send.id = "blog-ai-send";
    send.type = "submit";
    shell.appendChild(input);
    shell.appendChild(send);
    ui.chatForm.appendChild(shell);
    var hint = el("div", "bai-composer-hint");
    hint.appendChild(el("span", null, "Enter 发送 · Shift+Enter 换行"));
    var countWrap = el("span", "bai-count");
    ui.count = el("span", null, "0");
    ui.countMax = el("span", null, String(LIMITS.chat));
    countWrap.appendChild(ui.count);
    countWrap.appendChild(document.createTextNode(" / "));
    countWrap.appendChild(ui.countMax);
    hint.appendChild(countWrap);
    ui.chatForm.appendChild(hint);

    main.appendChild(ui.chatForm);

    app.appendChild(aside);
    app.appendChild(main);
    ui.overlay.appendChild(backdrop);
    ui.overlay.appendChild(app);

    root.appendChild(toggle);
    root.appendChild(ui.overlay);
    document.body.appendChild(root);

    toggle.addEventListener("click", function () {
      setOpen(!state.open);
      if (state.open) {
        updatePageHint();
        refreshAuthMe().then(refreshAllQuotas);
        setTimeout(function () { if (ui.input) ui.input.focus(); }, 120);
      }
    });

    ui.modeChat.addEventListener("click", function () {
      setMode("chat");
    });

    ui.modeImage.addEventListener("click", function () {
      setMode("image");
      if (ui.messages && !ui.messages.querySelector(".bai-image-intro")) {
        var intro = appendMsg(
          ui.messages,
          "system",
          state.isLogin
            ? "生图使用 Agnes Image，默认每天 3 张。"
            : "生图需要先登录，避免额度被刷。"
        );
        intro.classList.add("bai-image-intro");
      }
    });

    ui.chatForm.addEventListener("submit", function (e) {
      e.preventDefault();
      if (state.mode === "image") {
        generateImage(input, send);
      } else {
        sendMessage(input, send);
      }
    });

    input.addEventListener("input", updateCount);
    input.addEventListener("input", function () {
      input.style.height = "auto";
      input.style.height = Math.min(input.scrollHeight, 140) + "px";
    });
    input.addEventListener("keydown", function (e) {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        ui.chatForm.requestSubmit();
      }
    });

    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && state.open) setOpen(false);
    });

    window.addEventListener("popstate", function () {
      if (state.open) updatePageHint();
    });

    window.addEventListener("blog-ai-open", function (e) {
      setOpen(true);
      var detail = (e && e.detail) || {};
      if (detail.openAuth === true) {
        openAuthModal(detail.mode || "login");
      }
      refreshAuthMe().then(refreshAllQuotas);
    });

    window.addEventListener("blog-ai-image-result", function (e) {
      var detail = (e && e.detail) || {};
      var image = detail.image || detail;
      if (image && image.url) openImageModal(image);
    });

    updateAuthBar();
    refreshAuthMe();
    refreshAllQuotas();
  }

  function setOpen(open) {
    state.open = open;
    var toggle = document.getElementById("blog-ai-toggle");
    if (toggle) toggle.setAttribute("aria-expanded", open ? "true" : "false");
    if (!ui.overlay) return;
    ui.overlay.classList.toggle("is-open", open);
    ui.overlay.setAttribute("aria-hidden", open ? "false" : "true");
    if (open) {
      document.body.classList.add("bai-app-open");
    } else {
      document.body.classList.remove("bai-app-open");
    }
  }

  function updateQuotaLabel() {
    if (!ui.quota) return;
    if (state.mode === "image") {
      if (state.unlimited) {
        ui.quota.textContent = "生图剩余：无限（站长）";
        return;
      }
      var imageSuffix = state.isLogin ? "" : " · 登录后可生图";
      ui.quota.textContent =
        "生图剩余：" + state.imageRemaining + "/" + state.imageLimit + imageSuffix;
      return;
    }
    if (state.unlimited) {
      ui.quota.textContent = "今日剩余：无限（站长）";
      return;
    }
    var suffix = state.isLogin ? "" : " · 登录后 50 次/天";
    ui.quota.textContent =
      "今日剩余：" + state.remaining + "/" + state.limit + suffix;
  }

  function appendMsg(container, role, text) {
    var wrap = el("div", "bai-msg " + role);
    var avatar = el("div", "bai-msg-role");
    if (role === "user") {
      var url = userAvatarURL();
      if (url) {
        var uimg = el("img");
        uimg.src = url;
        uimg.alt = "我";
        avatar.appendChild(uimg);
        avatar.classList.add("is-clickable");
        avatar.title = "点击更换头像";
        avatar.addEventListener("click", pickAvatar);
      } else {
        avatar.textContent = userInitial();
      }
    } else if (role === "bot") {
      var bimg = el("img");
      bimg.src = LOGO_URL;
      bimg.alt = "AI";
      avatar.appendChild(bimg);
    }
    var bubble = el("div", "bai-msg-bubble", text);
    wrap.appendChild(avatar);
    wrap.appendChild(bubble);
    container.appendChild(wrap);
    container.scrollTop = container.scrollHeight;
    return wrap;
  }

  function ensureImageModal() {
    if (ui.imageOverlay) return;

    ui.imageOverlay = el("div", "blog-ai-image-overlay");
    ui.imageOverlay.id = "blog-ai-image-overlay";
    ui.imageOverlay.setAttribute("aria-hidden", "true");

    var backdrop = el("button", "blog-ai-image-backdrop");
    backdrop.type = "button";
    backdrop.setAttribute("aria-label", "关闭生成图片");
    backdrop.addEventListener("click", closeImageModal);

    ui.imageCard = el("div", "blog-ai-image-card");
    ui.imageCard.setAttribute("role", "dialog");
    ui.imageCard.setAttribute("aria-modal", "true");
    ui.imageCard.setAttribute("aria-labelledby", "blog-ai-image-title");

    var close = el("button", "blog-ai-image-close", "关闭");
    close.type = "button";
    close.addEventListener("click", closeImageModal);

    ui.imageMedia = el("div", "blog-ai-image-media");
    var img = document.createElement("img");
    ui.imagePreview = img;
    img.alt = "AI 生成图片";
    img.loading = "lazy";
    img.addEventListener("load", function () {
      var ratio =
        img.naturalWidth > 0 && img.naturalHeight > 0
          ? img.naturalWidth / img.naturalHeight
          : 1;
      ui.imageCard.style.setProperty("--blog-ai-result-aspect", ratio.toFixed(4));
      img.classList.add("is-loaded");
    });
    ui.imageMedia.appendChild(img);

    var copy = el("div", "blog-ai-image-copy");
    copy.appendChild(el("p", "blog-ai-image-kicker", "AI Image"));
    ui.imageTitle = el("h3", null, "生图完成");
    ui.imageTitle.id = "blog-ai-image-title";
    copy.appendChild(ui.imageTitle);

    var actions = el("div", "blog-ai-image-actions");
    ui.imageSave = el("button", "blog-ai-image-action", "保存本地");
    ui.imageSave.type = "button";
    actions.appendChild(ui.imageSave);
    ui.imageCopy = el("button", "blog-ai-image-action", "复制链接");
    ui.imageCopy.type = "button";
    actions.appendChild(ui.imageCopy);

    copy.appendChild(actions);
    ui.imageCard.appendChild(close);
    ui.imageCard.appendChild(ui.imageMedia);
    ui.imageCard.appendChild(copy);
    ui.imageOverlay.appendChild(backdrop);
    ui.imageOverlay.appendChild(ui.imageCard);
    document.body.appendChild(ui.imageOverlay);
  }

  function imageFilenameFromUrl(rawUrl) {
    try {
      var parsed = new URL(rawUrl, window.location.href);
      var filename = decodeURIComponent(
        (parsed.pathname.split("/").pop() || "").trim()
      );
      if (filename && /\.[a-z0-9]{2,5}$/i.test(filename)) return filename;
    } catch (err) {
      // Fall through to a stable default name.
    }
    return "taozhiyy-ai-image.png";
  }

  function triggerImageDownload(url, filename) {
    var link = document.createElement("a");
    link.href = url;
    link.download = filename;
    link.rel = "noopener noreferrer";
    document.body.appendChild(link);
    link.click();
    link.remove();
  }

  function saveImageLocally(rawUrl) {
    var filename = imageFilenameFromUrl(rawUrl);
    return fetch(rawUrl, { mode: "cors", credentials: "omit" })
      .then(function (res) {
        if (!res.ok) throw new Error("image download failed");
        return res.blob();
      })
      .then(function (blob) {
        var objectUrl = URL.createObjectURL(blob);
        triggerImageDownload(objectUrl, filename);
        setTimeout(function () {
          URL.revokeObjectURL(objectUrl);
        }, 1000);
        return "downloaded";
      })
      .catch(function () {
        window.open(rawUrl, "_blank", "noopener,noreferrer");
        return "opened";
      });
  }

  function openImageModal(image) {
    if (!image || !image.url) return;
    ensureImageModal();
    ui.imagePreview.classList.remove("is-loaded");
    ui.imagePreview.src = image.url;
    setTimeout(function () {
      if (ui.imagePreview && ui.imagePreview.complete) {
        ui.imagePreview.classList.add("is-loaded");
      }
    }, 0);
    ui.imageSave.textContent = "保存本地";
    ui.imageSave.onclick = function () {
      ui.imageSave.disabled = true;
      ui.imageSave.textContent = "保存中";
      saveImageLocally(image.url).then(function (result) {
        ui.imageSave.textContent = result === "opened" ? "已打开" : "已保存";
        setTimeout(function () {
          if (ui.imageSave) {
            ui.imageSave.disabled = false;
            ui.imageSave.textContent = "保存本地";
          }
        }, 1400);
      });
    };
    ui.imageCopy.textContent = "复制链接";
    ui.imageCopy.onclick = function () {
      if (!navigator.clipboard) return;
      navigator.clipboard.writeText(image.url).then(function () {
        ui.imageCopy.textContent = "已复制";
        setTimeout(function () {
          if (ui.imageCopy) ui.imageCopy.textContent = "复制链接";
        }, 1400);
      });
    };
    state.imageModalOpen = true;
    ui.imageOverlay.classList.add("is-open");
    ui.imageOverlay.setAttribute("aria-hidden", "false");
    document.body.classList.add("blog-ai-image-open");
  }

  function closeImageModal() {
    if (!ui.imageOverlay) return;
    state.imageModalOpen = false;
    ui.imageOverlay.classList.remove("is-open");
    ui.imageOverlay.setAttribute("aria-hidden", "true");
    document.body.classList.remove("blog-ai-image-open");
  }

  function applyImageQuota(data) {
    if (!data) return;
    if (data.limit != null) state.imageLimit = data.limit;
    if (data.used != null) state.imageUsed = data.used;
    if (data.remaining != null) state.imageRemaining = data.remaining;
    if (data.isLogin != null) state.isLogin = !!data.isLogin;
    if (data.unlimited != null) state.unlimited = !!data.unlimited;
    if (data.imageEnabled != null) state.imageEnabled = !!data.imageEnabled;
  }

  function refreshAllQuotas() {
    return Promise.all([refreshQuota(), refreshImageQuota()]).then(function () {
      updateQuotaLabel();
    });
  }

  function refreshQuota() {
    return fetch(API_CHAT, {
      method: "GET",
      credentials: "include",
      headers: { Accept: "application/json" },
    })
      .then(function (r) {
        return r.json().then(function (d) {
          return { ok: r.ok, data: d };
        });
      })
      .then(function (res) {
        if (res.ok && res.data) {
          state.limit = res.data.limit ?? state.limit;
          state.used = res.data.used ?? state.used;
          state.remaining = res.data.remaining ?? state.remaining;
          state.isLogin = !!res.data.isLogin;
          state.unlimited = !!res.data.unlimited;
          if (res.data.chatEnabled === false) state.chatEnabled = false;
        }
        updateQuotaLabel();
      })
      .catch(function () {
        updateQuotaLabel();
      });
  }

  function refreshImageQuota() {
    return fetch(API_IMAGE, {
      method: "GET",
      credentials: "include",
      headers: { Accept: "application/json" },
    })
      .then(function (r) {
        return r.json().then(function (d) {
          return { ok: r.ok, data: d };
        });
      })
      .then(function (res) {
        if (res.ok && res.data) {
          applyImageQuota(res.data);
        }
        updateQuotaLabel();
      })
      .catch(function () {
        updateQuotaLabel();
      });
  }

  function generateImage(input, sendBtn) {
    if (state.sending || state.authModalOpen) return;
    var text = (input.value || "").trim();
    if (!text) return;
    if (!state.isLogin) {
      appendMsg(ui.messages, "system", "请先登录后再生图，避免额度被刷爆。");
      openAuthModal("login");
      return;
    }
    if (!state.imageEnabled) {
      appendMsg(ui.messages, "system", IMAGE_NOT_CONFIGURED_MSG);
      return;
    }
    if (!state.unlimited && state.imageRemaining <= 0) {
      appendMsg(ui.messages, "system", IMAGE_LIMIT_MSG);
      return;
    }

    state.sending = true;
    sendBtn.disabled = true;
    input.disabled = true;
    appendMsg(ui.messages, "user", text);
    input.value = "";
    var thinking = appendMsg(ui.messages, "bot", "正在生成图片...");

    fetch(API_IMAGE, {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: JSON.stringify({ prompt: text, size: "1024*1024" }),
    })
      .then(function (r) {
        return r.json().then(function (d) {
          return { status: r.status, data: d };
        });
      })
      .then(function (res) {
        thinking.remove();
        var d = res.data || {};
        applyImageQuota(d);
        updateQuotaLabel();

        if (d.error === "IMAGE_NOT_CONFIGURED" || d.imageEnabled === false) {
          appendMsg(ui.messages, "system", d.message || IMAGE_NOT_CONFIGURED_MSG);
          return;
        }
        if (res.status === 401 || d.error === "LOGIN_REQUIRED") {
          appendMsg(ui.messages, "system", d.message || "请先登录后再生图。");
          openAuthModal("login");
          return;
        }
        if (res.status === 429 || d.error === "DAILY_LIMIT_EXCEEDED") {
          state.imageRemaining = 0;
          appendMsg(ui.messages, "system", d.message || IMAGE_LIMIT_MSG);
          return;
        }
        if (res.status >= 400 || d.error || !d.image || !d.image.url) {
          appendMsg(ui.messages, "system", d.message || IMAGE_ERROR_MSG);
          return;
        }
        appendMsg(ui.messages, "system", "图片已生成，已在屏幕中央打开。");
        openImageModal(d.image);
      })
      .catch(function () {
        thinking.remove();
        appendMsg(ui.messages, "system", IMAGE_ERROR_MSG);
      })
      .finally(function () {
        state.sending = false;
        sendBtn.disabled = false;
        input.disabled = false;
        input.focus();
      });
  }

  function sendMessage(input, sendBtn) {
    if (state.sending || state.authModalOpen) return;
    var text = (input.value || "").trim();
    if (!text) return;
    if (!state.chatEnabled) {
      appendMsg(ui.messages, "system", NOT_CONFIGURED_MSG);
      return;
    }
    if (!state.unlimited && state.remaining <= 0) {
      appendMsg(ui.messages, "system", LIMIT_MSG);
      return;
    }

    var ctx = getPageContext();
    state.sending = true;
    sendBtn.disabled = true;
    input.disabled = true;
    appendMsg(ui.messages, "user", text);
    input.value = "";
    var thinking = appendMsg(ui.messages, "bot", "思考中...");

    fetch(API_CHAT, {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json", Accept: "application/json" },
      body: JSON.stringify(
        window.BlogAIPageContext &&
          typeof window.BlogAIPageContext.createChatPayload === "function"
          ? window.BlogAIPageContext.createChatPayload(text, ctx)
          : {
              message: text,
              pageUrl: ctx.pageUrl,
              pageTitle: ctx.pageTitle,
              pageContext: ctx,
            }
      ),
    })
      .then(function (r) {
        return r.json().then(function (d) {
          return { status: r.status, data: d };
        });
      })
      .then(function (res) {
        thinking.remove();
        var d = res.data || {};

        if (d.limit != null) state.limit = d.limit;
        if (d.used != null) state.used = d.used;
        if (d.remaining != null) state.remaining = d.remaining;
        if (d.isLogin != null) state.isLogin = d.isLogin;
        if (d.unlimited != null) state.unlimited = d.unlimited;
        if (d.chatEnabled === false) state.chatEnabled = false;
        updateQuotaLabel();

        if (d.error === "CHAT_NOT_CONFIGURED" || d.chatEnabled === false) {
          appendMsg(ui.messages, "system", d.message || NOT_CONFIGURED_MSG);
          return;
        }
        if (res.status === 429 || d.error === "DAILY_LIMIT_EXCEEDED") {
          state.remaining = 0;
          appendMsg(ui.messages, "system", d.message || LIMIT_MSG);
          return;
        }
        if (res.status >= 400 || d.error) {
          appendMsg(ui.messages, "system", d.message || ERROR_MSG);
          return;
        }
        appendMsg(ui.messages, "bot", d.reply || "（没有收到回复）");
      })
      .catch(function () {
        thinking.remove();
        appendMsg(ui.messages, "system", ERROR_MSG);
      })
      .finally(function () {
        state.sending = false;
        sendBtn.disabled = false;
        input.disabled = false;
        input.focus();
      });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", mount);
  } else {
    mount();
  }
})();
