import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Link } from "react-router-dom";
import { FiBell, FiEye, FiEyeOff, FiRefreshCw, FiUpload, FiX } from "react-icons/fi";
import AvatarCropDialog from "../components/owner-console/AvatarCropDialog";
import { authLogin, authLogout, authUploadAvatar } from "../services/authApi";
import {
  approveOwnerFriend,
  createOwnerFixedAnswer,
  createOwnerDraft,
  deleteOwnerEntry,
  fetchMusicConfig,
  generateOwnerAi,
  isPublicImageURL,
  markOwnerNotificationRead,
  publishOwnerArticle,
  publishOwnerFriend,
  publishOwnerGalleryImage,
  publishOwnerJournal,
  publishOwnerMoment,
  saveMusicConfig,
  uploadOwnerAsset,
} from "../services/ownerApi";
import { parseFriendQuickInput } from "../lib/friendQuickInput";
import { notifyEntriesChanged } from "../lib/siteEntries";
import {
  AboutPanel,
  ArticleLibrary,
  BiliPanel,
  FriendLibrary,
  GalleryLibrary,
  MomentLibrary,
  ProjectPanel,
} from "./ownerLibrary";
import { ownerFriendPublishToast } from "../lib/ownerPublishMessages";
import {
  buildMobileArticleDraft,
  ownerConsoleAvatars,
  ownerConsoleModules,
  ownerConsoleScreens,
  publishSteps,
} from "../pwa/appConsoleBlueprint";
import {
  getBackendHealthLabel,
  getDeployLabel,
  getOwnerFixedAnswers,
  getOwnerGuestbookContacts,
  getOwnerSessionLabel,
  getOwnerRegisteredUsers,
  getStatsSnapshot,
} from "../pwa/appConsoleState";
import {
  getOwnerGalleryAlbumSelection,
  ownerCustomGalleryAlbumValue,
  ownerGalleryAlbumOptions,
} from "../lib/ownerGalleryAlbums";

const screenMap = Object.fromEntries(ownerConsoleScreens.map((screen) => [screen.id, screen]));

const defaultMobileMaterial =
  "今天把手机端 AI 发文流程接到真实草稿接口，图片和文字都能先保存到站长控制台。";

const todayMoment = () => {
  const now = new Date();
  return {
    year: String(now.getFullYear()),
    date: `${now.getMonth() + 1}.${now.getDate()}`,
  };
};

const defaultMoment = {
  ...todayMoment(),
  type: "碎碎念",
  content: "",
};

const initialCandidateAnswers = [
  "这是站长控制台里的 AI 调试区，满意答案会保存到后端固定问答库。",
  "固定答案保存后，网站右下角 AI 助手遇到相同问题会优先返回这条回复。",
  "如果没有命中固定答案，AI 助手会继续走 DeepSeek 正常聊天链路。",
];

const fetchWithin = (url, options, timeoutMs) => {
  const controller = new AbortController();
  const timer = window.setTimeout(() => controller.abort(), timeoutMs);
  return fetch(url, { ...options, signal: controller.signal }).finally(() => {
    window.clearTimeout(timer);
  });
};

const readJson = async (res) => {
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message || data.error || `HTTP ${res.status}`);
  return data;
};

const triggerBackendSync = async () => {
  const res = await fetch("/api/v1/sync/trigger", {
    method: "POST",
    headers: { Accept: "application/json" },
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) throw new Error(data.message || data.error || `HTTP ${res.status}`);
  return data;
};

const moduleToneClass = {
  blue: "owner-module-icon owner-module-icon--blue",
  green: "owner-module-icon owner-module-icon--green",
  rose: "owner-module-icon owner-module-icon--rose",
  sun: "owner-module-icon owner-module-icon--sun",
};

const StatusTag = ({ children }) => <span className="owner-tag">{children}</span>;

const draftDetail = (item) => {
  const kind = item.kind || "article";
  const updatedAt = item.updatedAt || item.createdAt || "";
  const bodyBytes = new TextEncoder().encode(item.body || "").length;
  const kindLabel = kind === "article" ? "文章" : kind;
  return `${kindLabel} · ${updatedAt || "刚刚"} · ${bodyBytes} 字节`;
};

const userInitial = (name) => (name || "?").slice(0, 1).toUpperCase();

const AVATAR_FILE_TYPES = new Set(["image/png", "image/jpeg", "image/webp", "image/gif"]);

const OwnerAvatarFace = ({ photo, fallback }) =>
  photo ? (
    <span className="owner-avatar-face owner-avatar-face--photo">
      <img src={photo} alt="" referrerPolicy="no-referrer" />
    </span>
  ) : (
    <span className="owner-avatar-face" style={{ background: fallback.gradient }}>
      {fallback.initial}
    </span>
  );

const notificationSourceLabel = (source) => {
  switch (source) {
    case "guestbook":
      return "留言板";
    case "friends":
      return "朋友页";
    default:
      return source || "提醒";
  }
};

const AppConsolePage = () => {
  // 后台界面不显示看板娘：挂载期间给 body 打标记，离开时还原。
  useEffect(() => {
    document.body.classList.add("owner-console-active");
    return () => document.body.classList.remove("owner-console-active");
  }, []);
  const [activeScreen, setActiveScreen] = useState("home");
  const [auth, setAuth] = useState(null);
  const [health, setHealth] = useState(null);
  const [stats, setStats] = useState(null);
  const [ownerStatus, setOwnerStatus] = useState(null);
  const [ownerEmails, setOwnerEmails] = useState(null);
  const [error, setError] = useState("");
  const [loginName, setLoginName] = useState("");
  const [loginPassword, setLoginPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [loginBusy, setLoginBusy] = useState(false);
  const [logoutBusy, setLogoutBusy] = useState(false);
  const [loginError, setLoginError] = useState("");
  const [notificationsOpen, setNotificationsOpen] = useState(false);
  const [avatarOpen, setAvatarOpen] = useState(false);
  const [avatar, setAvatar] = useState(ownerConsoleAvatars[0]);
  const [avatarUploadBusy, setAvatarUploadBusy] = useState(false);
  const [avatarCropFile, setAvatarCropFile] = useState(null);
  const [articleTitle, setArticleTitle] = useState("");
  const [articleKind, setArticleKind] = useState("博客");
  const [articleBody, setArticleBody] = useState("");
  const [articleImageURL, setArticleImageURL] = useState("");
  const [friendName, setFriendName] = useState("");
  const [friendDesc, setFriendDesc] = useState("");
  const [friendUrl, setFriendUrl] = useState("");
  const [friendAvatar, setFriendAvatar] = useState("");
  const [friendQuickInput, setFriendQuickInput] = useState("");
  const [momentYear, setMomentYear] = useState(defaultMoment.year);
  const [momentDate, setMomentDate] = useState(defaultMoment.date);
  const [momentType, setMomentType] = useState(defaultMoment.type);
  const [momentContent, setMomentContent] = useState(defaultMoment.content);
  const [mobileMaterial, setMobileMaterial] = useState(defaultMobileMaterial);
  const [mobileDraft, setMobileDraft] = useState({
    title: "待生成文章",
    body: "点击“让 AI 写草稿”后，这里会出现移动端文章草稿。",
  });
  const [mobileToast, setMobileToast] = useState(
    "把图片和文字交给 AI 后，第一阶段会先保存为真实草稿，不直接发线上。",
  );
  const [aiQuestion, setAiQuestion] = useState("网站 AI 助手是怎么做出来的？");
  const [candidateAnswers, setCandidateAnswers] = useState(initialCandidateAnswers);
  const [selectedAnswerIndex, setSelectedAnswerIndex] = useState(0);
  const [manualAnswer, setManualAnswer] = useState("");
  const [answerToast, setAnswerToast] = useState("");
  const [answerSaveBusy, setAnswerSaveBusy] = useState(false);
  const [galleryAlbum, setGalleryAlbum] = useState(ownerGalleryAlbumOptions[0]?.value || "");
  const [customGalleryAlbum, setCustomGalleryAlbum] = useState("");
  const [galleryURLInput, setGalleryURLInput] = useState("");
  const [galleryUploads, setGalleryUploads] = useState([]);
  const [saveBusy, setSaveBusy] = useState(false);
  const [publishBusy, setPublishBusy] = useState(false);
  const [articleUploadBusy, setArticleUploadBusy] = useState(false);
  const [uploadBusy, setUploadBusy] = useState(false);
  const [galleryPublishBusy, setGalleryPublishBusy] = useState(false);
  const [friendPublishBusy, setFriendPublishBusy] = useState(false);
  const [momentPublishBusy, setMomentPublishBusy] = useState(false);
  const [readingNotificationId, setReadingNotificationId] = useState(null);
  const [approvingId, setApprovingId] = useState(null);
  const [deploy, setDeploy] = useState(null);
  const [mobileAiBusy, setMobileAiBusy] = useState(false);
  const [answersBusy, setAnswersBusy] = useState(false);
  const [publishView, setPublishView] = useState(null);
  const articleTextareaRef = useRef(null);
  const [publishState, setPublishState] = useState({
    open: false,
    title: "发布任务",
    activeIndex: -1,
    failIndex: null,
    simulated: false,
    toast: "第一阶段主要提供真实上传和真实草稿，不直接改线上正式内容。",
    view: null,
  });
  const publishTimerRef = useRef(null);

  const readSession = useCallback(async () => {
    try {
      const res = await fetchWithin(
        "/api/auth/me",
        { credentials: "include", headers: { Accept: "application/json" } },
        4000,
      );
      const data = await res.json().catch(() => ({}));
      setAuth(res.ok ? data : { loggedIn: false });
    } catch {
      setAuth({ loggedIn: false });
    }
  }, []);

  const refreshSignals = useCallback(async () => {
    const [healthResult, statsResult, ownerStatusResult, ownerEmailsResult, deployResult] =
      await Promise.allSettled([
        fetchWithin("/api/v1/health", { headers: { Accept: "application/json" } }, 3000).then(
          readJson,
        ),
        fetchWithin("/api/chat/stats?days=14", { headers: { Accept: "application/json" } }, 3000).then(
          readJson,
        ),
        fetchWithin(
          "/api/owner/status",
          { credentials: "include", headers: { Accept: "application/json" } },
          8000,
        ).then(readJson),
        fetchWithin(
          "/api/owner/emails",
          { credentials: "include", headers: { Accept: "application/json" } },
          8000,
        ).then(readJson),
        fetchWithin(
          "/api/client/deploy-status",
          { headers: { Accept: "application/json" } },
          6000,
        ).then(readJson),
      ]);
    if (healthResult.status === "fulfilled") setHealth(healthResult.value);
    if (statsResult.status === "fulfilled") setStats(statsResult.value);
    if (ownerStatusResult.status === "fulfilled") setOwnerStatus(ownerStatusResult.value);
    if (ownerEmailsResult.status === "fulfilled") setOwnerEmails(ownerEmailsResult.value);
    if (deployResult.status === "fulfilled") setDeploy(deployResult.value);
  }, []);

  const loadConsole = useCallback(async () => {
    setError("");
    const session = readSession();
    const signals = refreshSignals();
    await session;
    await signals;
  }, [readSession, refreshSignals]);

  useEffect(() => {
    readSession();
    refreshSignals();
    return () => {
      if (publishTimerRef.current) clearInterval(publishTimerRef.current);
    };
  }, [readSession, refreshSignals]);

  const activeMeta = screenMap[activeScreen] || screenMap.home;
  const ownerLabel = getOwnerSessionLabel(auth);
  const savedAvatarUrl = isPublicImageURL(auth?.user?.avatar || "") ? auth.user.avatar.trim() : "";
  const healthLabel = getBackendHealthLabel(health);
  const statsSnapshot = useMemo(() => getStatsSnapshot(stats), [stats]);
  const liveNotifications = ownerStatus?.notifications?.items ?? [];
  const notificationTotal = ownerStatus?.notifications?.total ?? liveNotifications.length;
  const registeredUsers = getOwnerRegisteredUsers(ownerEmails);
  const guestbookContacts = getOwnerGuestbookContacts(ownerEmails);
  const registeredUserTotal =
    ownerStatus?.users?.registeredTotal ?? ownerStatus?.users?.total ?? registeredUsers.length;
  const fixedAnswers = getOwnerFixedAnswers(ownerStatus);
  const liveDrafts = ownerStatus?.drafts?.items || [];
  const draftCount = ownerStatus?.drafts?.total ?? liveDrafts.length;
  const friendPending = liveNotifications.filter((item) => item.source === "friends").length;
  const deployLabel = getDeployLabel(deploy);
  const deployRun = deploy?.available ? (deploy.runs || [])[0] : null;
  const moduleStatus = {
    article: `草稿 ${draftCount}`,
    gallery: `待上传 ${galleryUploads.length}`,
    moments: "轻量发布",
    friend: `申请 ${friendPending}`,
    inbox: `新消息 ${notificationTotal}`,
    emails: "后台可见",
  };
  const previewBody = useMemo(
    () => articleBody.replaceAll("---", "").trim().slice(0, 420) || "Markdown 预览",
    [articleBody],
  );

  const openScreen = (screenId) => {
    setActiveScreen(screenId);
    setNotificationsOpen(false);
    setAvatarOpen(false);
  };

  const [musicInput, setMusicInput] = useState("");
  const [musicEnabled, setMusicEnabled] = useState(false);
  const [musicBusy, setMusicBusy] = useState(false);
  const [musicNotice, setMusicNotice] = useState("");

  useEffect(() => {
    let cancelled = false;
    fetchMusicConfig()
      .then((data) => {
        if (cancelled) return;
        setMusicInput(data.raw || "");
        setMusicEnabled(!!data.enabled);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, []);

  const handleMusicSave = async () => {
    setMusicBusy(true);
    setMusicNotice("");
    try {
      const data = await saveMusicConfig(musicInput.trim());
      setMusicEnabled(!!data.enabled);
      setMusicInput(data.raw || "");
      setMusicNotice(
        data.enabled
          ? `已保存，前台将播放歌单 ${data.playlistId}（版权受限曲目自动跳过）`
          : "已停用歌单，前台回退站点默认音乐",
      );
      window.dispatchEvent(new CustomEvent("blog-music-config-updated"));
    } catch (e) {
      setMusicNotice(e.message || "保存失败");
    } finally {
      setMusicBusy(false);
    }
  };

  const [deleteKind, setDeleteKind] = useState("article");
  const [deleteId, setDeleteId] = useState("");
  const [deleteBusy, setDeleteBusy] = useState(false);
  const [deleteNotice, setDeleteNotice] = useState("");

  const handleDeleteEntry = async () => {
    const id = deleteId.trim();
    if (!/^\d+$/.test(id)) {
      setDeleteNotice("请输入内容 ID（纯数字，在各列表或发布结果里可见）。");
      return;
    }
    if (!window.confirm(`确认删除 ${deleteKind} 类型 ID ${id}？线上页面会同步移除。`)) return;
    setDeleteBusy(true);
    setDeleteNotice("");
    try {
      await deleteOwnerEntry(id, deleteKind);
      notifyEntriesChanged();
      await loadConsole();
      setDeleteId("");
      setDeleteNotice(`已删除 ${deleteKind} ID ${id}。`);
    } catch (e) {
      setDeleteNotice(e.message || "删除失败");
    } finally {
      setDeleteBusy(false);
    }
  };

  const clearPublishTimer = () => {
    if (publishTimerRef.current) {
      clearInterval(publishTimerRef.current);
      publishTimerRef.current = null;
    }
  };

  const startPublish = (title) => {
    clearPublishTimer();
    setPublishState({
      open: true,
      title,
      activeIndex: 0,
      failIndex: null,
      simulated: true,
      toast: "这里展示发布流程预览；真实写入能力已经接到草稿、图片和文章/相册发布接口。",
    });

    let step = 0;
    publishTimerRef.current = setInterval(() => {
      step += 1;
      if (step >= publishSteps.length) {
        clearPublishTimer();
        setPublishState((current) => ({
          ...current,
          activeIndex: publishSteps.length,
          failIndex: null,
          simulated: true,
          toast: "流程预览完成。真实发布请使用文章或相册里的发布按钮。",
        }));
        return;
      }
      setPublishState((current) => ({ ...current, activeIndex: step, failIndex: null, simulated: true }));
    }, 760);
  };

  const simulateFail = () => {
    clearPublishTimer();
    setPublishState((current) => ({
      ...current,
      open: true,
      activeIndex: 2,
      failIndex: 2,
      simulated: true,
      toast: "这里是失败状态预览，不会影响线上内容。",
    }));
  };

  const closePublish = () => {
    clearPublishTimer();
    setPublishState((current) => ({ ...current, open: false }));
  };

  const handleAvatarFile = (event) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;
    if (file.type && !AVATAR_FILE_TYPES.has(file.type)) {
      setError("仅支持 PNG / JPEG / WebP / GIF");
      return;
    }
    setError("");
    setAvatarOpen(false);
    setAvatarCropFile(file);
  };

  const handleAvatarCropConfirm = async (file) => {
    setAvatarUploadBusy(true);
    try {
      const result = await authUploadAvatar(file);
      if (!result.ok) {
        throw new Error(result.data?.message || "头像上传失败");
      }
      const nextUser = result.data?.user || {};
      setAuth((current) => ({
        ...(current || {}),
        loggedIn: true,
        user: { ...(current?.user || {}), ...nextUser },
      }));
      setAvatarCropFile(null);
      setError("");
      setPublishState({
        open: true,
        title: "头像已保存",
        activeIndex: -1,
        failIndex: null,
        simulated: false,
        view: null,
        toast: nextUser.avatar || "头像已写入对象存储",
      });
    } finally {
      setAvatarUploadBusy(false);
    }
  };

  const handleLogin = async (event) => {
    event.preventDefault();
    setLoginBusy(true);
    setLoginError("");
    try {
      const result = await authLogin(loginName.trim(), loginPassword);
      if (!result.ok) {
        setLoginError(
          result.data?.message || result.data?.error || "登录失败，请检查账号和密码",
        );
        return;
      }
      setAuth(result.data);
      setLoginPassword("");
      await loadConsole();
    } catch (e) {
      setLoginError(e.message || "登录失败，请稍后再试");
    } finally {
      setLoginBusy(false);
    }
  };

  const handleLogout = async () => {
    setLogoutBusy(true);
    setLoginError("");
    try {
      await authLogout();
    } catch {
      // 接口失败也清掉本地会话，避免停在已登录界面。
    } finally {
      setAuth({ loggedIn: false });
      setOwnerStatus(null);
      setOwnerEmails(null);
      setNotificationsOpen(false);
      setAvatarOpen(false);
      setLogoutBusy(false);
    }
  };

  const saveArticleDraft = async ({ title, body, toastTitle, toastMessage }) => {
    setSaveBusy(true);
    clearPublishTimer();
    try {
      await createOwnerDraft({
        kind: "article",
        title,
        body,
        coverUrl: articleImageURL.trim(),
        status: "draft",
      });
      setPublishState({
        open: true,
        title: toastTitle,
        activeIndex: -1,
        failIndex: null,
        simulated: false,
        toast: toastMessage,
      });
      await loadConsole();
    } catch (e) {
      setError(e.message || "草稿保存失败");
    } finally {
      setSaveBusy(false);
    }
  };

  const handleSaveDraft = async () => {
    await saveArticleDraft({
      title: articleTitle,
      body: articleBody,
      toastTitle: "草稿已保存",
      toastMessage: "桌面端 Markdown 草稿已保存到真实后端，后续可以继续编辑。",
    });
  };

  const handlePublishArticle = async ({
    draftId,
    title,
    body,
    coverUrl = "",
    publishTitle = "发布文章",
  }) => {
    if (!title.trim() || !body.trim()) {
      setError("请先填写标题和正文，再发布。");
      return;
    }
    setPublishBusy(true);
    clearPublishTimer();
    setError("");
    setPublishView(null);
    setPublishState({
      open: true,
      title: publishTitle,
      activeIndex: 3,
      failIndex: null,
      simulated: false,
      toast: "正在提交到真实文章发布接口...",
    });

    try {
      const data = articleKind === "建站记录"
        ? await publishOwnerJournal({ title, body, coverUrl })
        : await publishOwnerArticle({
            ...(draftId ? { draftId } : {}),
            title,
            body,
            coverUrl,
          });
      const item = data.item || {};
      setPublishView(
        item.id
          ? {
              to: articleKind === "建站记录" ? `/build/post/${item.id}` : `/blog/${item.id}`,
              label: articleKind === "建站记录" ? "查看建站日志" : "查看线上文章",
            }
          : null,
      );
      setPublishState({
        open: true,
        title: publishTitle,
        activeIndex: publishSteps.length,
        failIndex: null,
        simulated: false,
        toast: item.path
          ? `正文已写入数据库 ${item.path}。图片地址保存在同一条记录里。`
          : "文章已写入数据库。",
      });
      notifyEntriesChanged();
      await loadConsole();
    } catch (e) {
      const message = e.message || "文章发布失败";
      setError(message);
      setPublishState({
        open: true,
        title: publishTitle,
        activeIndex: 3,
        failIndex: 3,
        simulated: false,
        toast: message,
      });
    } finally {
      setPublishBusy(false);
    }
  };

  const generateMobileArticle = async () => {
    setMobileAiBusy(true);
    try {
      const data = await generateOwnerAi({ kind: "draft", input: mobileMaterial });
      setMobileDraft({ title: data.title || "AI 草稿", body: data.body || "" });
      setMobileToast(`AI 已生成待审核草稿（${data.model || "模型"}），确认后会保存到真实草稿箱。`);
    } catch (e) {
      const draft = buildMobileArticleDraft(mobileMaterial);
      setMobileDraft(draft);
      setMobileToast(`AI 接口不可用（${e.message || "请求失败"}），已改用本地模板草稿，请人工审核。`);
    } finally {
      setMobileAiBusy(false);
    }
  };

  const approveMobileArticle = async () => {
    const nextDraft =
      mobileDraft.title === "待生成文章" ? buildMobileArticleDraft(mobileMaterial) : mobileDraft;
    if (nextDraft !== mobileDraft) setMobileDraft(nextDraft);
    await saveArticleDraft({
      title: nextDraft.title,
      body: nextDraft.body,
      toastTitle: "移动端 AI 草稿已保存",
      toastMessage: "移动端 AI 文章已经保存为真实草稿，当前阶段不会直接发布到网站。",
    });
    setMobileToast("AI 草稿已保存到真实草稿箱，可以在草稿页继续处理。");
  };

  const handleGalleryUpload = async (event) => {
    const file = event.target.files?.[0];
    if (!file) return;
    setUploadBusy(true);
    setError("");
    try {
      const albumSelection = getOwnerGalleryAlbumSelection(galleryAlbum, customGalleryAlbum);
      const uploadAlbum = albumSelection.albumId || albumSelection.albumTitle;
      if (!uploadAlbum) {
        throw new Error("请选择相册，或填写自定义相册名。");
      }
      const data = await uploadOwnerAsset(file, { kind: "gallery", album: uploadAlbum });
      const item = data.item || {};
      setGalleryUploads((current) => [
        {
          name: item.path || item.name || file.name,
          url: item.url || "",
          albumId: albumSelection.albumId,
          albumTitle: albumSelection.albumTitle,
        },
        ...current,
      ]);
      setPublishState({
        open: true,
        title: "图片已上传",
        activeIndex: -1,
        failIndex: null,
        toast: `图片已上传到真实后端：${item.url || file.name}`,
      });
      await loadConsole();
    } catch (e) {
      setError(e.message || "图片上传失败");
    } finally {
      event.target.value = "";
      setUploadBusy(false);
    }
  };

  const handleAddGalleryURL = () => {
    const next = galleryURLInput.trim();
    if (!isPublicImageURL(next)) {
      setError("请输入有效的公开图片 URL。");
      return;
    }
    const albumSelection = getOwnerGalleryAlbumSelection(galleryAlbum, customGalleryAlbum);
    if (!albumSelection.albumId && !albumSelection.albumTitle) {
      setError("请选择相册，或填写自定义相册名。");
      return;
    }
    setError("");
    setGalleryUploads((current) => [
      {
        name: "PicGo 图片 URL",
        url: next,
        albumId: albumSelection.albumId,
        albumTitle: albumSelection.albumTitle,
      },
      ...current,
    ]);
    setGalleryURLInput("");
    setPublishState({
      open: true,
      title: "相册图片链接已加入",
      activeIndex: -1,
      failIndex: null,
      simulated: false,
      toast: `相册发布将使用 ${next}`,
    });
  };

  const handlePublishGalleryImage = async () => {
    const latest = galleryUploads[0];
    if (!latest?.url) {
      setError("请先上传图片，或添加一个公开图片 URL。");
      return;
    }
    if (!latest.albumId && !latest.albumTitle) {
      setError("请选择相册，或填写自定义相册名。");
      return;
    }

    setGalleryPublishBusy(true);
    clearPublishTimer();
    setError("");
    setPublishState({
      open: true,
      title: "发布相册图片",
      activeIndex: 3,
      failIndex: null,
      simulated: false,
      toast: "正在提交到真实相册发布接口...",
    });

    try {
      const data = await publishOwnerGalleryImage({
        albumId: latest.albumId || "",
        albumTitle: latest.albumTitle || "",
        imageUrl: latest.url,
      });
      const item = data.item || {};
      setGalleryUploads((current) =>
        current.map((entry, index) =>
          index === 0
            ? {
                ...entry,
                albumId: item.albumId || entry.albumId,
                published: true,
              }
            : entry,
        ),
      );
      const albumId = item.extra?.albumId || latest.albumId || item.id;
      setPublishView(albumId ? { to: `/gallery/${albumId}`, label: "查看线上相册" } : null);
      setPublishState({
        open: true,
        title: "发布相册图片",
        activeIndex: publishSteps.length,
        failIndex: null,
        simulated: false,
        toast: `图片地址已写入数据库 ${item.path || ""}`.trim(),
      });
      notifyEntriesChanged();
      await loadConsole();
    } catch (e) {
      const message = e.message || "相册发布失败";
      setError(message);
      setPublishState({
        open: true,
        title: "发布相册图片",
        activeIndex: 3,
        failIndex: 3,
        simulated: false,
        toast: message,
      });
    } finally {
      setGalleryPublishBusy(false);
    }
  };

  const handlePublishMoment = async () => {
    const year = momentYear.trim();
    const date = momentDate.trim();
    const type = momentType.trim();
    const content = momentContent.trim();
    if (!year || !type || !content) {
      setError("请填写碎语年份、分类和内容。");
      return;
    }

    setMomentPublishBusy(true);
    clearPublishTimer();
    setError("");
    setPublishState({
      open: true,
      title: "发布碎语",
      activeIndex: 3,
      failIndex: null,
      simulated: false,
      toast: "正在提交到真实碎语发布接口...",
    });

    try {
      const data = await publishOwnerMoment({ year, date, type, content });
      const item = data.item || {};
      setPublishView({ to: "/glimmer", label: "查看线上碎语" });
      setPublishState({
        open: true,
        title: "发布碎语",
        activeIndex: publishSteps.length,
        failIndex: null,
        simulated: false,
        toast: `碎语已写入数据库 ${item.path || ""}`.trim(),
      });
      notifyEntriesChanged();
      await loadConsole();
    } catch (e) {
      const message = e.message || "碎语发布失败";
      setError(message);
      setPublishState({
        open: true,
        title: "发布碎语",
        activeIndex: 3,
        failIndex: 3,
        simulated: false,
        toast: message,
      });
    } finally {
      setMomentPublishBusy(false);
    }
  };

  const handleFriendQuickFill = () => {
    const parsed = parseFriendQuickInput(friendQuickInput);
    if (!parsed.name || !parsed.desc || !parsed.url) {
      setError("没有识别完整，请确认四行里包含站名、描述和站点 URL。");
      return;
    }
    setFriendName(parsed.name);
    setFriendDesc(parsed.desc);
    setFriendUrl(parsed.url);
    setFriendAvatar(parsed.avatar);
    setError("");
    setPublishState({
      open: true,
      title: "友链快速识别",
      activeIndex: -1,
      failIndex: null,
      simulated: false,
      toast: "已按四行格式识别站名、描述、站点 URL 和头像 URL（本地规则），并填入友链表单。",
    });
  };

  const handlePublishFriend = async () => {
    const name = friendName.trim();
    const desc = friendDesc.trim();
    const url = friendUrl.trim();
    const avatarUrl = friendAvatar.trim();
    if (!name || !desc || !url) {
      setError("请填写站点名称、站点描述和站点 URL。");
      return;
    }
    if (!isPublicImageURL(url)) {
      setError("友链 URL 必须是公开的 http 或 https 地址。");
      return;
    }
    if (avatarUrl && !isPublicImageURL(avatarUrl)) {
      setError("友链头像 URL 必须是公开的 http 或 https 地址。");
      return;
    }

    setFriendPublishBusy(true);
    clearPublishTimer();
    setError("");
    setPublishState({
      open: true,
      title: "发布友链",
      activeIndex: 3,
      failIndex: null,
      simulated: false,
      toast: "正在提交到真实友链发布接口...",
    });

    try {
      const data = await publishOwnerFriend({ name, desc, url, avatar: avatarUrl });
      const item = data.item || {};
      setPublishView(
        item.pullRequestURL
          ? { href: item.pullRequestURL, label: "查看友链 PR" }
          : { to: "/friends", label: "查看朋友页" },
      );
      setFriendName("");
      setFriendDesc("");
      setFriendUrl("");
      setFriendAvatar("");
      setFriendQuickInput("");
      setPublishState({
        open: true,
        title: "发布友链",
        activeIndex: publishSteps.length,
        failIndex: null,
        simulated: false,
        toast: ownerFriendPublishToast(item),
      });
      await loadConsole();
    } catch (e) {
      const message = e.message || "友链发布失败";
      setError(message);
      setPublishState({
        open: true,
        title: "发布友链",
        activeIndex: 3,
        failIndex: 3,
        simulated: false,
        toast: message,
      });
    } finally {
      setFriendPublishBusy(false);
    }
  };

  const handleApproveFriend = async (item) => {
    if (!item?.id) {
      setError("这条申请没有可处理的 ID，请刷新后再试。");
      return;
    }
    setApprovingId(item.id);
    setError("");
    try {
      const data = await approveOwnerFriend(item.id);
      notifyEntriesChanged();
      await loadConsole();
      const name = data.item?.name || "友链";
      setPublishView({ to: "/friends", label: "查看朋友页" });
      setPublishState({
        open: true,
        title: "友链已通过",
        activeIndex: -1,
        failIndex: null,
        simulated: false,
        view: null,
        toast: data.duplicate
          ? `${name} 已在友链列表里，未重复添加。${data.githubNote || ""}`
          : `${name} 已展示到朋友页。${data.githubNote || ""}`,
      });
    } catch (e) {
      setError(e.message || "通过失败");
    } finally {
      setApprovingId(null);
    }
  };

  const handleMarkNotificationRead = async (item) => {
    if (!item?.id) {
      setError("这条提醒没有可标记的留言 ID，请刷新后再试。");
      return;
    }
    setReadingNotificationId(item.id);
    setError("");
    try {
      await markOwnerNotificationRead(item.id);
      await loadConsole();
      setPublishState({
        open: true,
        title: "提醒已读",
        activeIndex: -1,
        failIndex: null,
        simulated: false,
        toast: "已从站长收件箱移除这条未读提醒，公开留言内容不会被隐藏。",
      });
    } catch (e) {
      setError(e.message || "标记已读失败");
    } finally {
      setReadingNotificationId(null);
    }
  };

  const applyMarkdownTool = (tool) => {
    const textarea = articleTextareaRef.current;
    if (!textarea) return;
    const start = textarea.selectionStart ?? articleBody.length;
    const end = textarea.selectionEnd ?? start;
    const selected = articleBody.slice(start, end);
    const wrap = (before, after, placeholder) => {
      const inner = selected || placeholder;
      return { text: before + inner + after, cursor: start + before.length + inner.length };
    };
    let next;
    switch (tool) {
      case "H1": {
        const lineStart = articleBody.lastIndexOf("\n", start - 1) + 1;
        const insert = "# ";
        next = {
          text: articleBody.slice(0, lineStart) + insert + articleBody.slice(lineStart),
          cursor: end + insert.length,
        };
        break;
      }
      case "B":
        next = wrap("**", "**", "加粗文字");
        break;
      case "I":
        next = wrap("*", "*", "斜体文字");
        break;
      case "Link":
        next = wrap("[", "](https://)", selected || "链接文字");
        break;
      case "Img":
        next = wrap("![", "](https://)", selected || "图片描述");
        break;
      case "Code":
        next = selected.includes("\n")
          ? wrap("```\n", "\n```", "代码")
          : wrap("`", "`", "代码");
        break;
      default:
        return;
    }
    setArticleBody(next.text);
    window.requestAnimationFrame(() => {
      textarea.focus();
      textarea.setSelectionRange(next.cursor, next.cursor);
    });
  };

  const handleArticleImageUpload = async (event) => {
    const file = event.target.files?.[0];
    if (!file) return;
    setArticleUploadBusy(true);
    setError("");
    try {
      const data = await uploadOwnerAsset(file, {
        kind: articleKind === "建站记录" ? "journal" : "article",
      });
      const item = data.item || {};
      const nextURL = item.url || "";
      setArticleImageURL(nextURL);
      setPublishState({
        open: true,
        title: "文章封面已上传",
        activeIndex: -1,
        failIndex: null,
        simulated: false,
        toast: nextURL || file.name,
      });
    } catch (e) {
      setError(e.message || "文章图片上传失败");
    } finally {
      event.target.value = "";
      setArticleUploadBusy(false);
    }
  };

  const testAiQuestion = async () => {
    const question = aiQuestion.trim();
    if (!question) {
      setAnswerToast("请先填写测试问题。");
      return;
    }
    setAnswersBusy(true);
    try {
      const data = await generateOwnerAi({ kind: "answers", input: question });
      const answers = Array.isArray(data.answers) ? data.answers.filter(Boolean) : [];
      if (!answers.length) throw new Error("模型没有返回候选答案");
      setCandidateAnswers(answers.slice(0, 3));
      setSelectedAnswerIndex(0);
      setAnswerToast(`AI 已生成 ${Math.min(answers.length, 3)} 条候选答案（${data.model || "模型"}）。`);
    } catch (e) {
      setCandidateAnswers([
        `关于“${question}”，可以先在控制台内调试回答，再保存为网站 AI 的固定回复。`,
        '保存后的固定答案会进入后端库，用户侧 AI 助手遇到同样问题会优先命中。',
        '没有命中固定答案的问题会继续走模型聊天，不影响原来的 AI 使用流程。',
      ]);
      setSelectedAnswerIndex(0);
      setAnswerToast(`AI 接口不可用（${e.message || "请求失败"}），已改用模板候选，请人工改写后保存。`);
    } finally {
      setAnswersBusy(false);
    }
  };

  const saveFixedAnswer = async () => {
    const question = aiQuestion.trim() || "未命名问题";
    const answer = manualAnswer.trim() || candidateAnswers[selectedAnswerIndex] || "";
    if (!answer) {
      setAnswerToast("请先选一条候选答案，或手动填写最终答案。");
      return;
    }
    setAnswerSaveBusy(true);
    setError("");
    try {
      await createOwnerFixedAnswer({ question, answer });
      setManualAnswer("");
      setAnswerToast("已同步到后端固定答案库，网站 AI 会优先命中这条回复。");
      await loadConsole();
    } catch (e) {
      setAnswerToast(e.message || "固定答案保存失败。");
    } finally {
      setAnswerSaveBusy(false);
    }
  };

  const handleSync = async () => {
    try {
      await triggerBackendSync();
      startPublish("刷新 Bili 缓存");
    } catch (e) {
      setError(e.message || "同步触发失败");
      simulateFail();
    }
  };

  const visibleDrafts = liveDrafts;

  // 站长登录界面：没登录只给账号密码表单，登录后才进控制台。
  if (!auth) {
    return (
      <main className="owner-login-page">
        <section className="owner-login-card" aria-busy="true">
          <header className="owner-login-head">
            <span className="owner-login-mark" aria-hidden="true">
              TC
            </span>
            <span className="owner-login-brand">
              <strong>Spume控制器</strong>
              <small>站长工作区</small>
            </span>
          </header>
          <p className="owner-login-kicker">Owner Console</p>
          <h1 className="owner-login-title">站长控制台</h1>
          <p className="owner-login-verse">桃之夭夭，灼灼其华。</p>
          <p className="owner-login-hint">
            <span className="owner-login-busy-dot" aria-hidden="true" />
            正在读取会话…
          </p>
        </section>
      </main>
    );
  }
  if (!auth.loggedIn) {
    return (
      <main className="owner-login-page">
        <form className="owner-login-card" onSubmit={handleLogin}>
          <header className="owner-login-head">
            <span className="owner-login-mark" aria-hidden="true">
              TC
            </span>
            <span className="owner-login-brand">
              <strong>Spume控制器</strong>
              <small>站长工作区</small>
            </span>
          </header>
          <p className="owner-login-kicker">Owner Console</p>
          <h1 className="owner-login-title">站长控制台</h1>
          <p className="owner-login-verse">桃之夭夭，灼灼其华。</p>
          <p className="owner-login-hint">用站长账号登录后发布文章、管理站点。</p>
          <div className="owner-login-rule" aria-hidden="true" />
          <div className="owner-login-field">
            <label className="owner-login-label" htmlFor="owner-login-name">
              账号
            </label>
            <input
              id="owner-login-name"
              className="owner-login-input"
              value={loginName}
              onChange={(event) => setLoginName(event.target.value)}
              placeholder="admin"
              autoComplete="username"
              autoFocus
            />
          </div>
          <div className="owner-login-field">
            <label className="owner-login-label" htmlFor="owner-login-password">
              密码
            </label>
            <div className="owner-login-input-wrap">
              <input
                id="owner-login-password"
                className="owner-login-input"
                type={showPassword ? "text" : "password"}
                value={loginPassword}
                onChange={(event) => setLoginPassword(event.target.value)}
                autoComplete="current-password"
              />
              <button
                type="button"
                className="owner-login-eye"
                onClick={() => setShowPassword((visible) => !visible)}
                aria-label={showPassword ? "隐藏密码" : "显示密码"}
                aria-pressed={showPassword}
              >
                {showPassword ? <FiEyeOff aria-hidden /> : <FiEye aria-hidden />}
              </button>
            </div>
          </div>
          {loginError ? (
            <p className="owner-login-error" role="alert">
              {loginError}
            </p>
          ) : null}
          <button type="submit" className="owner-login-submit" disabled={loginBusy}>
            {loginBusy ? "登录中…" : "登录"}
          </button>
          <Link className="owner-login-back" to="/">
            返回首页
          </Link>
        </form>
      </main>
    );
  }

  return (
    <main className="owner-console">
      <div className="owner-console-app">
        <aside className="owner-console-sidebar owner-glass" aria-label="站长控制器分区">
          <button
            type="button"
            className="owner-brand"
            onClick={() => openScreen("home")}
            aria-label="打开站长控制器"
          >
            <span className="owner-brand-mark">TC</span>
            <span>
              <strong>Spume控制器</strong>
              <small>站长工作区</small>
            </span>
          </button>
          <nav className="owner-nav">
            {ownerConsoleScreens.map((screen) => (
              <button
                type="button"
                key={screen.id}
                className={activeScreen === screen.id ? "active" : ""}
                onClick={() => openScreen(screen.id)}
              >
                <span className="owner-nav-icon">{screen.icon}</span>
                <span>{screen.navLabel}</span>
              </button>
            ))}
          </nav>
          <div className="owner-session-actions">
            <Link className="owner-secondary owner-small-button" to="/">
              回到主页
            </Link>
            <button
              type="button"
              className="owner-secondary owner-small-button"
              onClick={handleLogout}
              disabled={logoutBusy}
            >
              {logoutBusy ? "退出中…" : "退出登录"}
            </button>
          </div>
        </aside>

        <section className="owner-workspace">
          <header className="owner-topbar owner-glass">
            <div>
              <h1 className="owner-page-title">{activeMeta.title}</h1>
              <p className="owner-page-sub">{activeMeta.subtitle}</p>
            </div>
            <div className="owner-top-actions">
              <div className="owner-session-actions">
                <Link className="owner-secondary owner-small-button" to="/">
                  回到主页
                </Link>
                <button
                  type="button"
                  className="owner-secondary owner-small-button"
                  onClick={handleLogout}
                  disabled={logoutBusy}
                >
                  {logoutBusy ? "退出中…" : "退出登录"}
                </button>
              </div>
              {deployRun?.url ? (
                <a
                  className="owner-pill"
                  href={deployRun.url}
                  target="_blank"
                  rel="noreferrer"
                  title="查看 GitHub 部署运行"
                >
                  <span className="owner-status-dot" />
                  最新部署：{deployLabel}
                </a>
              ) : (
                <span className="owner-pill">
                  <span className="owner-status-dot" />
                  最新部署：{deployLabel}
                </span>
              )}
              <div className="owner-notify-wrap">
                <button
                  type="button"
                  className="owner-icon-button"
                  onClick={() => {
                    setNotificationsOpen((open) => !open);
                    setAvatarOpen(false);
                  }}
                  aria-label="消息提醒"
                >
                  <FiBell aria-hidden />
                  <span className="owner-badge">{notificationTotal}</span>
                </button>
                {notificationsOpen ? (
                  <div className="owner-popover owner-glass">
                    <div className="owner-panel-title">
                      <h2>消息提醒</h2>
                      <StatusTag>{notificationTotal}</StatusTag>
                    </div>
                    <div className="owner-notice-list">
                      {liveNotifications.length ? null : (
                        <p className="owner-record-empty">还没有未读提醒。</p>
                      )}
                      {liveNotifications.map((item) => (
                        <button
                          type="button"
                          key={item.id || `${item.source}-${item.title}`}
                          className="owner-notice"
                          onClick={() => openScreen("inbox")}
                        >
                          <span className="owner-dot" />
                          <span>
                            <strong>{item.title}</strong>
                            <small>{item.detail}</small>
                          </span>
                        </button>
                      ))}
                    </div>
                  </div>
                ) : null}
              </div>
              <div className="owner-avatar-wrap">
                <button
                  type="button"
                  className="owner-avatar-button"
                  onClick={() => {
                    setAvatarOpen((open) => !open);
                    setNotificationsOpen(false);
                  }}
                  aria-label="更换头像"
                >
                  <OwnerAvatarFace photo={savedAvatarUrl} fallback={avatar} />
                </button>
                {avatarOpen ? (
                  <div className="owner-avatar-switcher owner-glass">
                    <div className="owner-panel-title">
                      <h2>更换头像</h2>
                      <StatusTag>{savedAvatarUrl ? "已保存" : "未设置"}</StatusTag>
                    </div>
                    <p className="owner-avatar-note">
                      支持 PNG、JPEG、WebP、GIF。选图后可以自由裁切，再写入对象存储。
                    </p>
                    <div className="owner-avatar-grid">
                      <label
                        className={`owner-avatar-option owner-avatar-upload${avatarUploadBusy ? " is-busy" : ""}`}
                        htmlFor="ownerAvatarUpload"
                      >
                        <FiUpload aria-hidden />
                        {avatarUploadBusy ? "上传中..." : "上传头像"}
                      </label>
                      <input
                        id="ownerAvatarUpload"
                        type="file"
                        accept="image/png,image/jpeg,image/webp,image/gif"
                        onChange={handleAvatarFile}
                        disabled={avatarUploadBusy}
                        hidden
                      />
                      {savedAvatarUrl
                        ? null
                        : ownerConsoleAvatars.map((item) => (
                            <button
                              type="button"
                              key={item.id}
                              className={`owner-avatar-option ${item.id === avatar.id ? "active" : ""}`}
                              onClick={() => {
                                setAvatar(item);
                                setAvatarOpen(false);
                              }}
                            >
                              <OwnerAvatarFace fallback={item} />
                              <span>{item.label}</span>
                            </button>
                          ))}
                    </div>
                    {savedAvatarUrl ? <p className="owner-avatar-url">{savedAvatarUrl}</p> : null}
                  </div>
                ) : null}
              </div>
            </div>
          </header>

          {error ? <p className="owner-alert">{error}</p> : null}

          <section className={`owner-screen ${activeScreen === "home" ? "active" : ""}`}>
            <div className="owner-content-grid">
              <div className="owner-home-main">
                <div className="owner-section-title">
                  <div>
                    <h2>快捷模块</h2>
                    <p>第一阶段重点接通真实数据、上传和草稿。</p>
                  </div>
                  <StatusTag>第一阶段</StatusTag>
                </div>
                <div className="owner-module-grid">
                  {ownerConsoleModules.map((module) => (
                    <button
                      type="button"
                      key={module.id}
                      className="owner-module-card"
                      onClick={() => openScreen(module.id)}
                    >
                      <span className="owner-module-head">
                        <span>
                          <strong>{module.title}</strong>
                          <small>{module.description}</small>
                        </span>
                        <span className={moduleToneClass[module.tone]}>{module.icon}</span>
                      </span>
                      <span className="owner-tiny-status">{moduleStatus[module.id] || ""}</span>
                    </button>
                  ))}
                </div>

                <section className="owner-panel owner-glass">
                  <div className="owner-panel-title">
                    <h2>用户与流量</h2>
                    <StatusTag>{ownerStatus ? "实时" : "备用"}</StatusTag>
                  </div>
                  <div className="owner-stats-grid">
                    <article className="owner-stat-card">
                      <span>注册用户</span>
                      <strong>{registeredUserTotal}</strong>
                      <em>真实数据</em>
                    </article>
                    <article className="owner-stat-card">
                      <span>站长会话</span>
                      <strong>{ownerLabel}</strong>
                      <em>{healthLabel}</em>
                    </article>
                    <article className="owner-stat-card">
                      <span>消息提醒</span>
                      <strong>{notificationTotal}</strong>
                      <em>guestbook + friends</em>
                    </article>
                    <article className="owner-stat-card">
                      <span>今日 AI</span>
                      <strong>{statsSnapshot.today}</strong>
                      <em>{statsSnapshot.model}</em>
                    </article>
                  </div>
                </section>
              </div>

              <aside className="owner-side-stack">
                <section className="owner-panel owner-glass">
                  <div className="owner-panel-title">
                    <h2>消息提醒</h2>
                    <StatusTag>{notificationTotal}</StatusTag>
                  </div>
                  <div className="owner-notice-list">
                    {liveNotifications.length ? null : (
                      <p className="owner-record-empty">还没有未读提醒。</p>
                    )}
                    {liveNotifications.map((item) => (
                      <button
                        type="button"
                        key={item.id || `${item.source}-${item.title}-panel`}
                        className="owner-notice"
                        onClick={() => openScreen("inbox")}
                      >
                        <span className="owner-dot" />
                        <span>
                          <strong>{item.title}</strong>
                          <small>{item.detail}</small>
                        </span>
                      </button>
                    ))}
                  </div>
                </section>

                <section className="owner-panel owner-glass">
                  <div className="owner-panel-title">
                    <h2>发布任务</h2>
                    <StatusTag>第一阶段</StatusTag>
                  </div>
                  <div className="owner-task-list">
                    <div className="owner-task-row">
                      <span>当前草稿</span>
                      <strong>{draftCount}</strong>
                    </div>
                    <div className="owner-task-row">
                      <span>已上传图片</span>
                      <strong>{galleryUploads.length}</strong>
                    </div>
                    <div className="owner-task-row">
                      <span>部署</span>
                      <strong>{deployLabel}</strong>
                    </div>
                  </div>
                  <button type="button" className="owner-secondary" onClick={handleSync}>
                    <FiRefreshCw aria-hidden /> 刷新后端状态
                  </button>
                </section>

                <section className="owner-panel owner-glass">
                  <div className="owner-panel-title">
                    <h2>背景音乐</h2>
                    <StatusTag>{musicEnabled ? "歌单播放" : "默认音乐"}</StatusTag>
                  </div>
                  <div className="owner-field">
                    <label htmlFor="musicPlaylist">网易云歌单链接或 ID</label>
                    <input
                      id="musicPlaylist"
                      value={musicInput}
                      placeholder="https://music.163.com/playlist?id=… 或数字 ID"
                      onChange={(event) => setMusicInput(event.target.value)}
                      disabled={musicBusy}
                    />
                  </div>
                  <p className="owner-frontmatter-hint">
                    保存后导航栏音乐按钮播放该歌单；版权/VIP 曲目自动跳过，清空保存则回退默认音乐。
                  </p>
                  {musicNotice ? <p className="owner-frontmatter-hint">{musicNotice}</p> : null}
                  <button
                    type="button"
                    className="owner-primary"
                    onClick={handleMusicSave}
                    disabled={musicBusy}
                  >
                    {musicBusy ? "保存中..." : "保存歌单配置"}
                  </button>
                </section>

                <section className="owner-panel owner-glass">
                  <div className="owner-panel-title">
                    <h2>删除内容</h2>
                    <StatusTag>按 ID</StatusTag>
                  </div>
                  <div className="owner-field">
                    <label htmlFor="deleteKind">类型</label>
                    <select
                      id="deleteKind"
                      value={deleteKind}
                      onChange={(event) => setDeleteKind(event.target.value)}
                      disabled={deleteBusy}
                    >
                      <option value="article">文章</option>
                      <option value="journal">建站日志</option>
                      <option value="gallery">相册图片</option>
                      <option value="moment">碎语</option>
                      <option value="about">关于我</option>
                      <option value="bili">哔哩视频</option>
                      <option value="project">星空馆</option>
                      <option value="friend">友链</option>
                    </select>
                  </div>
                  <div className="owner-field">
                    <label htmlFor="deleteId">内容 ID</label>
                    <input
                      id="deleteId"
                      value={deleteId}
                      placeholder="纯数字 ID"
                      onChange={(event) => setDeleteId(event.target.value)}
                      disabled={deleteBusy}
                    />
                  </div>
                  <p className="owner-frontmatter-hint">
                    各分区列表自带删除按钮；这里可按 ID 删除任意类型，ID 见列表或发布结果。
                  </p>
                  {deleteNotice ? <p className="owner-frontmatter-hint">{deleteNotice}</p> : null}
                  <button
                    type="button"
                    className="owner-secondary"
                    onClick={handleDeleteEntry}
                    disabled={deleteBusy}
                  >
                    {deleteBusy ? "删除中..." : "删除该内容"}
                  </button>
                </section>
              </aside>
            </div>
          </section>

          <section className={`owner-screen ${activeScreen === "article" ? "active" : ""}`}>
            <div className="owner-form-shell owner-glass owner-desktop-article">
              <div>
                <div className="owner-field">
                  <label htmlFor="articleType">文章类型</label>
                  <select
                    id="articleType"
                    value={articleKind}
                    onChange={(event) => setArticleKind(event.target.value)}
                  >
                    <option value="博客">博客</option>
                    <option value="建站记录">建站记录</option>
                  </select>
                </div>
                <div className="owner-field">
                  <label htmlFor="articleTitle">标题</label>
                  <input
                    id="articleTitle"
                    value={articleTitle}
                    placeholder="给文章起个标题"
                    onChange={(e) => setArticleTitle(e.target.value)}
                  />
                </div>
                <p className="owner-frontmatter-hint">
                  当前阶段会把 Markdown 草稿保存到真实后端，文章发布也会提交到真实站长接口。
                </p>
                <div className="owner-field">
                  <label htmlFor="articleCoverUrl">封面图片 URL</label>
                  <input
                    id="articleCoverUrl"
                    value={articleImageURL}
                    placeholder="https://cdn.example/article-cover.png"
                    onChange={(e) => setArticleImageURL(e.target.value)}
                  />
                </div>
                <div className="owner-quick-line">
                  <label className="owner-secondary" htmlFor="articleImageUpload">
                    <FiUpload aria-hidden /> {articleUploadBusy ? "上传中..." : "上传封面到 S3"}
                  </label>
                  <input
                    id="articleImageUpload"
                    type="file"
                    accept="image/png,image/jpeg,image/webp,image/gif"
                    onChange={handleArticleImageUpload}
                    hidden
                  />
                  <span className="owner-pill">
                    {articleImageURL
                      ? "封面已准备好发布"
                      : "粘贴 PicGo URL，或上传一张封面图"}
                  </span>
                </div>
                <div className="owner-md-editor">
                  <div className="owner-md-toolbar">
                    {["H1", "B", "I", "Link", "Img", "Code"].map((tool) => (
                      <button
                        type="button"
                        key={tool}
                        title={`插入 ${tool} 标记`}
                        onClick={() => applyMarkdownTool(tool)}
                      >
                        {tool}
                      </button>
                    ))}
                    <span>.md</span>
                  </div>
                  <textarea
                    ref={articleTextareaRef}
                    value={articleBody}
                    placeholder="在这里写 Markdown 正文，或到移动端用 AI 生成草稿。"
                    onChange={(e) => setArticleBody(e.target.value)}
                  />
                </div>
                <div className="owner-quick-line">
                  <button
                    type="button"
                    className="owner-secondary"
                    onClick={handleSaveDraft}
                    disabled={saveBusy}
                  >
                    {saveBusy ? "保存中..." : "保存草稿"}
                  </button>
                  <button
                    type="button"
                    className="owner-primary"
                    onClick={() =>
                      handlePublishArticle({
                        title: articleTitle,
                        body: articleBody,
                        coverUrl: articleImageURL.trim(),
                        publishTitle: "发布文章",
                      })
                    }
                    disabled={publishBusy}
                  >
                    {publishBusy ? "发布中..." : articleKind === "建站记录" ? "发布建站日志" : "发布文章"}
                  </button>
                </div>
              </div>
              <aside className="owner-md-preview">
                <div className="owner-kicker">Markdown 预览</div>
                <div className="owner-cover">{articleImageURL ? "封面已链接" : "封面预览"}</div>
                <h2>{articleTitle || "未命名文章"}</h2>
                <pre>{previewBody}</pre>
                <p>
                  {articleImageURL
                    ? `封面 URL：${articleImageURL}`
                    : "添加公开封面 URL，或上传一张封面到 S3。"}
                </p>
              </aside>
            </div>

            <div className="owner-mobile-article">
              <section className="owner-agent-card owner-glass">
                <div className="owner-panel-title">
                  <h2>移动端 AI 草稿</h2>
                  <StatusTag>共用草稿接口</StatusTag>
                </div>
                <div className="owner-agent-steps">
                  <div>
                    <b>1. 添加素材</b>
                    <span>写下要点或手机端随手记录。</span>
                  </div>
                  <div>
                    <b>2. 生成 AI 草稿</b>
                    <span>先保留当前草稿生成流程。</span>
                  </div>
                  <div>
                    <b>3. 保存真实草稿</b>
                    <span>确认后会写入同一个后端草稿箱。</span>
                  </div>
                </div>
                <div className="owner-agent-dropzone">
                  <strong>第一阶段说明</strong>
                  <span>移动端内容已经接到真实草稿层，发布层请从草稿箱继续处理。</span>
                </div>
                <div className="owner-field">
                  <label htmlFor="mobileArticleInput">AI 素材</label>
                  <textarea
                    id="mobileArticleInput"
                    value={mobileMaterial}
                    onChange={(e) => setMobileMaterial(e.target.value)}
                  />
                </div>
                <div className="owner-quick-line">
                  <button
                    type="button"
                    className="owner-secondary"
                    onClick={generateMobileArticle}
                    disabled={mobileAiBusy}
                  >
                    {mobileAiBusy ? "AI 生成中..." : "生成 AI 草稿"}
                  </button>
                  <button type="button" className="owner-primary" onClick={approveMobileArticle}>
                    保存确认后的草稿
                  </button>
                </div>
                <p className="owner-agent-toast">{mobileToast}</p>
              </section>
              <aside className="owner-agent-preview owner-glass">
                <div className="owner-kicker">AI 草稿预览</div>
                <div className="owner-cover">素材封面</div>
                <h2>{mobileDraft.title}</h2>
                <pre>{mobileDraft.body}</pre>
              </aside>
            </div>
            <ArticleLibrary />
          </section>

          <section className={`owner-screen ${activeScreen === "drafts" ? "active" : ""}`}>
            <div className="owner-panel owner-glass">
              <div className="owner-panel-title">
                <h2>草稿箱</h2>
                <button
                  type="button"
                  className="owner-primary owner-small-button"
                  onClick={() => openScreen("article")}
                >
                  新建文章
                </button>
              </div>
              <div className="owner-mini-metrics">
                <div>
                  <span>文章草稿</span>
                  <b>{draftCount}</b>
                </div>
                <div>
                  <span>已上传图片</span>
                  <b>{galleryUploads.length}</b>
                </div>
                <div>
                  <span>缺少封面</span>
                  <b>{visibleDrafts.filter((item) => !item.coverUrl).length}</b>
                </div>
                <div>
                  <span>待处理</span>
                  <b>{visibleDrafts.length}</b>
                </div>
              </div>
              <div className="owner-draft-list">
                {visibleDrafts.length ? null : (
                  <p className="owner-record-empty">
                    草稿箱是空的。去「发文章」写一篇，保存草稿后会出现在这里。
                  </p>
                )}
                {visibleDrafts.map((item, index) => (
                  <article className="owner-draft-item" key={item.id || `${item.title}-${index}`}>
                    <span>
                      <strong>{item.title}</strong>
                      <small>{draftDetail(item)}</small>
                    </span>
                    <span className="owner-quick-line">
                      <button
                        type="button"
                        className="owner-secondary"
                        onClick={() => {
                          setArticleTitle(item.title || "");
                          setArticleBody(item.body || "");
                          setArticleImageURL(item.coverUrl || "");
                          openScreen("article");
                        }}
                      >
                        继续编辑
                      </button>
                      <button
                        type="button"
                        className="owner-primary"
                        onClick={() =>
                          handlePublishArticle({
                            draftId: item.id,
                            title: item.title || articleTitle,
                            body: item.body || articleBody,
                            coverUrl: item.coverUrl || "",
                            publishTitle: "草稿发布流程",
                          })
                        }
                        disabled={publishBusy}
                      >
                        {publishBusy ? "发布中..." : "立即发布"}
                      </button>
                    </span>
                  </article>
                ))}
              </div>
            </div>
          </section>

          <section className={`owner-screen ${activeScreen === "gallery" ? "active" : ""}`}>
            <div className="owner-form-shell owner-glass">
              <div>
                <div className="owner-field">
                  <label htmlFor="galleryAlbum">相册</label>
                  <select
                    id="galleryAlbum"
                    value={galleryAlbum}
                    onChange={(e) => setGalleryAlbum(e.target.value)}
                  >
                    {ownerGalleryAlbumOptions.map((option) => (
                      <option key={option.value} value={option.value}>
                        {option.label}
                      </option>
                    ))}
                  </select>
                </div>
                {galleryAlbum === ownerCustomGalleryAlbumValue ? (
                  <div className="owner-field owner-field--spaced">
                    <label htmlFor="customGalleryAlbum">自定义相册</label>
                    <input
                      id="customGalleryAlbum"
                      value={customGalleryAlbum}
                      placeholder="请输入自定义相册名"
                      onChange={(e) => setCustomGalleryAlbum(e.target.value)}
                    />
                  </div>
                ) : null}
                <label className="owner-dropzone" htmlFor="galleryUpload">
                  <strong>选择要上传的图片</strong>
                  <span>
                    上传图片到 COS，或在下方粘贴公开 PicGo 图片 URL。
                  </span>
                </label>
                <input
                  id="galleryUpload"
                  type="file"
                  accept="image/png,image/jpeg,image/webp,image/gif"
                  onChange={handleGalleryUpload}
                  hidden
                />
                <div className="owner-field owner-field--spaced">
                  <label htmlFor="galleryURLInput">公开图片 URL</label>
                  <input
                    id="galleryURLInput"
                    value={galleryURLInput}
                    placeholder="https://cdn.example/gallery/demo.png"
                    onChange={(e) => setGalleryURLInput(e.target.value)}
                  />
                </div>
                <div className="owner-thumb-grid">
                  {(galleryUploads.length ? galleryUploads : [{ name: "暂无上传" }]).map((item) => (
                    <span key={item.url || item.name}>
                      {item.name}
                      {item.albumTitle ? ` -> ${item.albumTitle}` : ""}
                    </span>
                  ))}
                </div>
                <div className="owner-quick-line">
                  <label
                    className={`owner-secondary ${uploadBusy ? "owner-secondary--disabled" : ""}`}
                    htmlFor="galleryUpload"
                    aria-disabled={uploadBusy}
                  >
                    <FiUpload aria-hidden /> {uploadBusy ? "上传中..." : "上传到 S3"}
                  </label>
                  <button type="button" className="owner-secondary" onClick={handleAddGalleryURL}>
                    添加 PicGo URL
                  </button>
                  <button
                    type="button"
                    className="owner-primary"
                    onClick={handlePublishGalleryImage}
                    disabled={!galleryUploads[0]?.url || galleryPublishBusy}
                  >
                    {galleryPublishBusy ? "发布中..." : "发布到相册"}
                  </button>
                </div>
              </div>
              <aside className="owner-preview-card">
                <div className="owner-kicker">上传预览</div>
                <div className="owner-cover">相册封面</div>
                <h2>{galleryUploads[0]?.name || "暂无已上传图片"}</h2>
                <p>
                  {galleryUploads[0]?.url
                    ? `真实上传 URL：${galleryUploads[0].url}。目标相册：${
                        galleryUploads[0].albumTitle || galleryUploads[0].albumId || "相册"
                      }。`
                    : "上传图片到 S3，或添加一个公开图片 URL。"}
                </p>
              </aside>
            </div>
            <GalleryLibrary />
          </section>

          <section className={`owner-screen ${activeScreen === "moments" ? "active" : ""}`}>
            <div className="owner-form-shell owner-glass">
              <div>
                <div className="owner-field">
                  <label htmlFor="momentYear">年份</label>
                  <input
                    id="momentYear"
                    value={momentYear}
                    placeholder="2026"
                    onChange={(e) => setMomentYear(e.target.value)}
                  />
                </div>
                <div className="owner-field">
                  <label htmlFor="momentDate">日期</label>
                  <input
                    id="momentDate"
                    value={momentDate}
                    placeholder="6.8"
                    onChange={(e) => setMomentDate(e.target.value)}
                  />
                </div>
                <div className="owner-field">
                  <label htmlFor="momentType">分类</label>
                  <input
                    id="momentType"
                    value={momentType}
                    placeholder="碎碎念"
                    onChange={(e) => setMomentType(e.target.value)}
                  />
                </div>
                <div className="owner-field">
                  <label htmlFor="momentContent">内容</label>
                  <textarea
                    id="momentContent"
                    value={momentContent}
                    placeholder="写下今天的一点点心情"
                    onChange={(e) => setMomentContent(e.target.value)}
                  />
                </div>
                <div className="owner-quick-line">
                  <button
                    type="button"
                    className="owner-primary"
                    onClick={handlePublishMoment}
                    disabled={momentPublishBusy}
                  >
                    {momentPublishBusy ? "发布中..." : "发布碎语"}
                  </button>
                </div>
              </div>
              <aside className="owner-preview-card">
                <div className="owner-kicker">碎语预览</div>
                <div className="owner-friend-preview">
                  <span>{momentYear || "2026"} · {momentDate || "6.8"}</span>
                  <div>
                    <h2>{momentType || "分类"}</h2>
                    <p>{momentContent || "碎语内容"}</p>
                  </div>
                </div>
                <p>发布后会写入首页右上角导航进入的“碎语”页面。</p>
              </aside>
            </div>
            <MomentLibrary />
          </section>

          <section className={`owner-screen ${activeScreen === "about" ? "active" : ""}`}>
            <AboutPanel />
          </section>

          <section className={`owner-screen ${activeScreen === "bili" ? "active" : ""}`}>
            <BiliPanel />
          </section>

          <section className={`owner-screen ${activeScreen === "projects" ? "active" : ""}`}>
            <ProjectPanel />
          </section>

          <section className={`owner-screen ${activeScreen === "friend" ? "active" : ""}`}>
            <div className="owner-form-shell owner-glass">
              <div>
                <div className="owner-field">
                  <label htmlFor="friendQuickInput">四行友链快速识别</label>
                  <textarea
                    id="friendQuickInput"
                    value={friendQuickInput}
                    placeholder={"把站名、描述、站点 URL、头像 URL 四行一起粘进来，顺序可以乱。"}
                    onChange={(e) => setFriendQuickInput(e.target.value)}
                  />
                </div>
                <div className="owner-quick-line">
                  <button type="button" className="owner-secondary" onClick={handleFriendQuickFill}>
                    智能识别填表
                  </button>
                </div>
                <div className="owner-field">
                  <label htmlFor="friendName">站点名称</label>
                  <input
                    id="friendName"
                    value={friendName}
                    onChange={(e) => setFriendName(e.target.value)}
                  />
                </div>
                <div className="owner-field">
                  <label htmlFor="friendUrl">站点 URL</label>
                  <input
                    id="friendUrl"
                    value={friendUrl}
                    onChange={(e) => setFriendUrl(e.target.value)}
                  />
                </div>
                <div className="owner-field">
                  <label htmlFor="friendAvatar">头像 URL</label>
                  <input
                    id="friendAvatar"
                    value={friendAvatar}
                    onChange={(e) => setFriendAvatar(e.target.value)}
                  />
                </div>
                <div className="owner-field">
                  <label htmlFor="friendDesc">站点描述</label>
                  <textarea
                    id="friendDesc"
                    value={friendDesc}
                    onChange={(e) => setFriendDesc(e.target.value)}
                  />
                </div>
                <div className="owner-quick-line">
                  <button
                    type="button"
                    className="owner-secondary"
                    onClick={() => {
                      if (!isPublicImageURL(friendUrl.trim())) {
                        setError("友链 URL 必须是公开的 http 或 https 地址。");
                        return;
                      }
                      if (friendAvatar.trim() && !isPublicImageURL(friendAvatar.trim())) {
                        setError("友链头像 URL 必须是公开的 http 或 https 地址。");
                        return;
                      }
                      setError("");
                      setPublishState({
                        open: true,
                        title: "友链校验通过",
                        activeIndex: -1,
                        failIndex: null,
                        simulated: false,
                        toast: "站点 URL 格式有效，可以提交到真实友链数据。",
                      });
                    }}
                  >
                    校验链接
                  </button>
                  <button
                    type="button"
                    className="owner-primary"
                    onClick={handlePublishFriend}
                    disabled={friendPublishBusy}
                  >
                    {friendPublishBusy ? "发布中..." : "发布友链"}
                  </button>
                </div>
              </div>
              <aside className="owner-preview-card">
                <div className="owner-kicker">友链卡片</div>
                <div className="owner-friend-preview">
                  {isPublicImageURL(friendAvatar.trim()) ? (
                    <img src={friendAvatar.trim()} alt={friendName || "站点头像"} />
                  ) : (
                    <span>{userInitial(friendName)}</span>
                  )}
                  <div>
                    <h2>{friendName || "站点名称"}</h2>
                    <p>{friendDesc || "站点描述"}</p>
                  </div>
                </div>
                <p>{friendUrl || "https://www.miku.coffee"}</p>
              </aside>
            </div>
            <FriendLibrary />
          </section>

          <section className={`owner-screen ${activeScreen === "inbox" ? "active" : ""}`}>
            <div className="owner-panel owner-glass">
              <div className="owner-panel-title">
                <h2>统一收件箱</h2>
                <StatusTag>{notificationTotal} 条提醒</StatusTag>
              </div>
              <div className="owner-inbox-list">
                {liveNotifications.length ? null : (
                  <p className="owner-record-empty">还没有未读提醒。新的留言和朋友页评论会出现在这里。</p>
                )}
                {liveNotifications.map((item) => (
                  <article className="owner-message" key={item.id || `${item.source}-${item.title}-inbox`}>
                    <div className="owner-message-head">
                      <strong>{item.title}</strong>
                      <StatusTag>{notificationSourceLabel(item.source)}</StatusTag>
                    </div>
                    <p>{item.detail}</p>
                    {item.content ? (
                      <blockquote className="owner-message-content">{item.content}</blockquote>
                    ) : null}
                    {item.source === "friends" && item.extra?.url ? (
                      <p className="owner-frontmatter-hint">
                        申请友链：{item.extra.name || item.extra.url} · {item.extra.url}
                        {item.extra.desc ? ` · ${item.extra.desc}` : ""}
                      </p>
                    ) : null}
                    {item.nickname || item.createdAt ? (
                      <small className="owner-message-meta">
                        {item.nickname || "访客"} · {item.createdAt || "时间未知"}
                      </small>
                    ) : null}
                    <div className="owner-quick-line">
                      <Link
                        className="owner-secondary"
                        to={item.source === "friends" ? "/friends#friends-guestbook" : "/guestbook"}
                        onClick={() => setNotificationsOpen(false)}
                      >
                        稍后回复
                      </Link>
                      <button
                        type="button"
                        className="owner-secondary"
                        onClick={() => handleMarkNotificationRead(item)}
                        disabled={readingNotificationId === item.id}
                      >
                        {readingNotificationId === item.id ? "处理中..." : "标记已读"}
                      </button>
                      {item.source === "friends" ? (
                        <>
                          <button
                            type="button"
                            className="owner-primary"
                            onClick={() => handleApproveFriend(item)}
                            disabled={approvingId === item.id}
                          >
                            {approvingId === item.id ? "通过中..." : "通过"}
                          </button>
                          <button
                            type="button"
                            className="owner-secondary"
                            onClick={() => openScreen("friend")}
                          >
                            手动填写
                          </button>
                        </>
                      ) : null}
                    </div>
                  </article>
                ))}
              </div>
            </div>
          </section>

          <section className={`owner-screen ${activeScreen === "emails" ? "active" : ""}`}>
            <div className="owner-content-grid">
              <section className="owner-panel owner-glass">
                <div className="owner-panel-title">
                  <h2>注册用户邮箱</h2>
                  <StatusTag>{registeredUsers.length}</StatusTag>
                </div>
                {registeredUsers.length ? (
                  <div className="owner-table-wrap">
                    <table className="owner-data-table">
                      <thead>
                        <tr>
                          <th>昵称</th>
                          <th>邮箱</th>
                          <th>注册时间</th>
                        </tr>
                      </thead>
                      <tbody>
                        {registeredUsers.map((user) => (
                          <tr key={`${user.email}-${user.createdAt}`}>
                            <td>{user.displayName || "未设置昵称"}</td>
                            <td>{user.email}</td>
                            <td>{user.createdAt || "未知"}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                ) : (
                  <article className="owner-empty-row">暂无注册用户邮箱</article>
                )}
              </section>

              <aside className="owner-side-stack">
                <section className="owner-panel owner-glass">
                  <div className="owner-panel-title">
                    <h2>留言联系邮箱</h2>
                    <StatusTag>{guestbookContacts.length}</StatusTag>
                  </div>
                  <div className="owner-user-list">
                    {guestbookContacts.length ? (
                      guestbookContacts.map((contact) => (
                        <article
                          className="owner-user-row owner-user-row--stacked"
                          key={`${contact.id}-${contact.contactEmail}`}
                        >
                          <span className="owner-user-avatar">{userInitial(contact.nickname || contact.contactEmail)}</span>
                          <span>
                            <strong>{contact.nickname || "访客"}</strong>
                            <small>{contact.contactEmail}</small>
                            {contact.accountEmail && contact.accountEmail !== contact.contactEmail ? (
                              <small>登录邮箱：{contact.accountEmail}</small>
                            ) : null}
                            {contact.content ? <small>{contact.content}</small> : null}
                          </span>
                          <StatusTag>{notificationSourceLabel(contact.source)}</StatusTag>
                        </article>
                      ))
                    ) : (
                      <article className="owner-empty-row">暂无留言联系邮箱</article>
                    )}
                  </div>
                </section>
              </aside>
            </div>
          </section>

          <section className={`owner-screen ${activeScreen === "ai" ? "active" : ""}`}>
            <div className="owner-panel owner-glass">
              <div className="owner-panel-title">
                <h2>AI 使用情况</h2>
                <StatusTag>今天</StatusTag>
              </div>
              <div className="owner-mini-metrics">
                <div>
                  <span>今天</span>
                  <b>{statsSnapshot.today}</b>
                </div>
                <div>
                  <span>周期</span>
                  <b>{statsSnapshot.period}</b>
                </div>
                <div>
                  <span>成功率</span>
                  <b>{statsSnapshot.successRate}</b>
                </div>
                <div>
                  <span>已配置</span>
                  <b>{statsSnapshot.configured ? "是" : "否"}</b>
                </div>
              </div>
            </div>

            <div className="owner-ai-workbench">
              <section className="owner-debug-box owner-glass">
                <div className="owner-panel-title">
                  <h2>AI 调试区</h2>
                  <StatusTag>后端同步</StatusTag>
                </div>
                <div className="owner-field">
                  <label htmlFor="aiQuestion">测试问题</label>
                  <textarea
                    id="aiQuestion"
                    value={aiQuestion}
                    onChange={(e) => setAiQuestion(e.target.value)}
                  />
                </div>
                <div className="owner-quick-line">
                  <button
                    type="button"
                    className="owner-secondary"
                    onClick={testAiQuestion}
                    disabled={answersBusy}
                  >
                    {answersBusy ? "AI 生成中..." : "生成候选答案"}
                  </button>
                  <button
                    type="button"
                    className="owner-primary"
                    onClick={saveFixedAnswer}
                    disabled={answerSaveBusy}
                  >
                    {answerSaveBusy ? "保存中..." : "保存固定答案"}
                  </button>
                </div>
                <div className="owner-panel-title owner-panel-title--spaced">
                  <h2>候选答案</h2>
                  <StatusTag>选择一条</StatusTag>
                </div>
                <div className="owner-candidate-list">
                  {candidateAnswers.map((answer, index) => (
                    <button
                      type="button"
                      key={answer}
                      className={selectedAnswerIndex === index ? "selected" : ""}
                      onClick={() => {
                        setSelectedAnswerIndex(index);
                        setAnswerToast("已选择一条候选答案。");
                      }}
                    >
                      {answer}
                    </button>
                  ))}
                </div>
                <div className="owner-field owner-field--spaced">
                  <label htmlFor="manualAnswer">手动最终答案</label>
                  <textarea
                    id="manualAnswer"
                    placeholder="在这里改写或补全最终答案。"
                    value={manualAnswer}
                    onChange={(e) => setManualAnswer(e.target.value)}
                  />
                </div>
                <p className="owner-answer-toast">{answerToast}</p>
              </section>

              <aside className="owner-debug-box owner-glass">
                <div className="owner-panel-title">
                  <h2>已保存答案</h2>
                  <StatusTag>后端固定答案</StatusTag>
                </div>
                <div className="owner-fixed-list">
                  {fixedAnswers.length ? (
                    fixedAnswers.map((item) => (
                      <article key={item.id || `${item.question}-${item.answer}`}>
                        <strong>{item.question}</strong>
                        <p>{item.answer}</p>
                      </article>
                    ))
                  ) : (
                    <article className="owner-empty-row">暂无后端固定答案</article>
                  )}
                </div>
              </aside>
            </div>
          </section>
        </section>
      </div>

      <nav className="owner-bottom-nav" aria-label="移动端站长控制器分区">
        {ownerConsoleScreens
          .filter((screen) => screen.id !== "friend")
          .map((screen) => (
            <button
              type="button"
              key={screen.id}
              className={activeScreen === screen.id ? "active" : ""}
              onClick={() => openScreen(screen.id)}
            >
              {screen.navLabel}
            </button>
          ))}
      </nav>

      <div className={`owner-drawer ${publishState.open ? "open" : ""}`}>
        <div className="owner-drawer-panel owner-glass">
          <div className="owner-drawer-head">
            <div>
              <div className="owner-kicker">发布流程</div>
              <h2>{publishState.title}</h2>
            </div>
            <button type="button" className="owner-close" onClick={closePublish} aria-label="关闭">
              <FiX aria-hidden />
            </button>
          </div>
          <div className="owner-step-list">
            {publishSteps.map((step, index) => {
              const done = index < publishState.activeIndex;
              const active = index === publishState.activeIndex;
              const failed = index === publishState.failIndex;
              return (
                <div
                  key={step}
                  className={[
                    "owner-step",
                    done ? "done" : "",
                    active ? "active" : "",
                    failed ? "fail" : "",
                  ]
                    .filter(Boolean)
                    .join(" ")}
                >
                  <span className="owner-step-no">{index + 1}</span>
                  <span>
                    <strong>{step}</strong>
                    <small>
                      {failed
                        ? publishState.simulated
                          ? "预览失败"
                          : "失败"
                        : active
                          ? "运行中"
                          : done
                            ? "已完成"
                            : "等待中"}
                    </small>
                  </span>
                  <em>{failed ? "失败" : done ? "完成" : active ? "进行中" : "等待"}</em>
                </div>
              );
            })}
          </div>
          <p className="owner-toast">{publishState.toast}</p>
          {publishView ? (
            <div className="owner-quick-line">
              {publishView.href ? (
                <a
                  className="owner-primary"
                  href={publishView.href}
                  target="_blank"
                  rel="noreferrer"
                >
                  {publishView.label}
                </a>
              ) : (
                <Link className="owner-primary" to={publishView.to} onClick={closePublish}>
                  {publishView.label}
                </Link>
              )}
            </div>
          ) : null}
          {publishState.simulated ? (
            <div className="owner-quick-line">
              <button type="button" className="owner-secondary" onClick={simulateFail}>
                预览失败
              </button>
              <button
                type="button"
                className="owner-primary"
                onClick={() => startPublish(publishState.title || "重试发布流程")}
              >
                重试预览
              </button>
            </div>
          ) : null}
        </div>
      </div>
      {avatarCropFile ? (
        <AvatarCropDialog
          file={avatarCropFile}
          busy={avatarUploadBusy}
          onCancel={() => {
            if (!avatarUploadBusy) setAvatarCropFile(null);
          }}
          onConfirm={handleAvatarCropConfirm}
        />
      ) : null}
    </main>
  );
};

export default AppConsolePage;
