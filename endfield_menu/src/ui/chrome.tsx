import { memo, useEffect, useState } from "react";
import { DISC_BUTTONS, PLUS_MARKS, SLIDES, YELLOW_RING } from "../data";
import type { Phase } from "../scene/engine";
import { GlyphText } from "./glyphs";
import { IconBolt, IconGear, IconMail, IconPower, IconSignal, IconWarn, IconX } from "./icons";

const CROSS_GAP = "M-14.5 -2.1h11.9v4.2h-11.9zM2.6 -2.1h11.9v4.2h-11.9zM-2.1 -14.5h4.2v11.9h-4.2zM-2.1 2.6h4.2v11.9h-4.2z";
const CROSS_SOLID = "M-11.85 -1h23v4.4h-23zM-2.55 -10.3h4.4v23h-4.4z";

export const SysButtons = memo(function SysButtons() {
  return (
    <div className="sys">
      <button type="button" data-action="settings" aria-label="设置"><IconGear /></button>
      <button type="button" data-action="mail" aria-label="邮件"><IconMail /></button>
      <button type="button" data-action="alerts" aria-label="警报"><IconWarn /><i className="dot" /></button>
    </div>
  );
});

export const PowerButton = memo(function PowerButton() {
  return (
    <button type="button" className="power" data-action="close" aria-label="关闭菜单">
      <i />
      <IconPower />
    </button>
  );
});

export function CloseButton() {
  return (
    <button type="button" className="close" data-action="close" aria-label="关闭">
      <span><IconX strokeWidth={4} /></span>
    </button>
  );
}

export function NetInfo() {
  return (
    <div className="net">
      <IconSignal />
      <span>82ms</span>
      <span>UID: xxxxx</span>
    </div>
  );
}

type SiteProfile = { avatar: string; displayName: string };

const AVATAR_UPDATED_EVENT = "blog-avatar-updated";
const AVATAR_REV_KEY = "spume-avatar-rev";

async function readSiteProfile(): Promise<SiteProfile> {
  try {
    const res = await fetch("/api/site/avatar", {
      headers: { Accept: "application/json" },
      cache: "no-store",
    });
    if (!res.ok) return { avatar: "", displayName: "" };
    const data = await res.json();
    const avatar = typeof data?.avatar === "string" ? data.avatar.trim() : "";
    const displayName = typeof data?.displayName === "string" ? data.displayName.trim() : "";
    const validAvatar = /^https?:\/\//.test(avatar) ? avatar : "";
    return { avatar: validAvatar, displayName };
  } catch {
    return { avatar: "", displayName: "" };
  }
}

export const Badge = memo(function Badge() {
  const [profile, setProfile] = useState<SiteProfile>({ avatar: "", displayName: "" });
  const [avatarFailed, setAvatarFailed] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const load = () => {
      readSiteProfile().then((next) => {
        if (!cancelled) {
          setProfile(next);
          setAvatarFailed(false);
        }
      });
    };
    load();
    const onStorage = (event: StorageEvent) => {
      if (event.key === AVATAR_REV_KEY) load();
    };
    window.addEventListener(AVATAR_UPDATED_EVENT, load);
    window.addEventListener("storage", onStorage);
    return () => {
      cancelled = true;
      window.removeEventListener(AVATAR_UPDATED_EVENT, load);
      window.removeEventListener("storage", onStorage);
    };
  }, []);

  const showAvatar = profile.avatar && !avatarFailed;

  return (
    <div className="badge">
      <div className="badge-plate" />
      <div className="badge-streak" />
      <div className="badge-rim" />
      <div className="badge-inner">
        <div className="avatar">
          {showAvatar ? (
            <img
              className="avatar-img"
              src={profile.avatar}
              alt="站长头像"
              onError={() => setAvatarFailed(true)}
            />
          ) : (
            "avatar"
          )}
          <i className="scan" /><i className="chip" />
        </div>
        <div className="mid">
          <div className="name">{profile.displayName || "xxxxx"}</div>
          <div className="sub">[ X · · · ]</div>
          <div className="lv">60</div>
          <div className="bar" />
          <div className="uid">UID:xxxxx</div>
        </div>
        <div className="perm">
          <i className="pill" />
          <div className="perm-txt"><small>权限等阶</small><b>满级</b></div>
          <svg viewBox="0 0 60 56" aria-hidden="true">
            <use href="#art-badge-tier" />
            <path d="M39.15 22.55L49.35 28.45L39.15 34.35Z" />
          </svg>
        </div>
      </div>
    </div>
  );
});

export function LevelPill() {
  return (
    <div className="level">
      <i className="lvl-bar" />
      <svg className="lvl-icon" viewBox="10 6 46 30" aria-hidden="true"><use href="#art-level-radar" /></svg>
      <span>探索等级</span>
      <i className="lvl-hatch" />
      <b>07</b>
      <button type="button" className="lvl-info" data-action="level-info" aria-label="说明" />
    </div>
  );
}

export function Energy() {
  return (
    <div className="energy">
      <span className="bolt"><IconBolt /></span>
      <b>7200</b><small>/9800</small>
      <b className="pct">100</b><small className="unit">%</small>
      <span className="cell"><IconBolt /></span>
    </div>
  );
}

export const TopBar = memo(function TopBar() {
  return (
    <div className="topbar">
      <div className="cur cur-a">
        <svg className="badge-a" viewBox="-26 -24 52 48" aria-hidden="true"><use href="#art-counter-a" /></svg>
        <b>355</b>
        <button type="button" className="plus-btn" data-action="topup-a" aria-label="获取" style={{ left: 121.63, top: 1.18 }}>
          <svg viewBox="-22 -22 44 44"><path d={CROSS_GAP} /></svg>
        </button>
      </div>
      <div className="cur cur-b">
        <svg className="badge-b" viewBox="-26 -26 52 52" aria-hidden="true"><use href="#art-counter-b" /></svg>
        <b>123040</b>
        <button type="button" className="plus-btn" data-action="topup-b" aria-label="获取" style={{ left: 172.34, top: 1.22 }}>
          <svg viewBox="-22 -22 44 44"><path d={CROSS_GAP} /></svg>
        </button>
      </div>
      <div className="stamina">
        <span className="stam-pill" />
        <svg className="badge-s" viewBox="-50 -50 105 95" aria-hidden="true"><use href="#art-stamina" /></svg>
        <b>178/360</b>
        <i />
        <button type="button" className="plus-btn solid" data-action="stamina" aria-label="获取" style={{ left: 236.2, top: -6.3 }}>
          <svg viewBox="-24 -24 48 48"><path d={CROSS_SOLID} /></svg>
        </button>
      </div>
    </div>
  );
});

export function News({ phase }: { phase: Phase }) {
  const [index, setIndex] = useState(0);
  useEffect(() => {
    if (phase !== "open") return;
    const id = window.setInterval(() => setIndex((i) => (i + 1) % SLIDES.length), 8000);
    return () => window.clearInterval(id);
  }, [phase]);
  const slide = SLIDES[index];
  return (
    <div className="news">
      <div className="news-frame" role="button" tabIndex={0} aria-label="Breaking News" onClick={() => setIndex((i) => (i + 1) % SLIDES.length)} onKeyDown={(e) => { if (e.key === "Enter") setIndex((i) => (i + 1) % SLIDES.length); }}>
        <div className="news-head">Breaking News
          <svg viewBox="0 0 12 15" aria-hidden="true"><path d="M1.5 2 6 5.6 10.5 2" fill="none" stroke="#fff" strokeWidth="1.5" /><path d="M2 8.4h8L6 13.6Z" fill="#fff" /></svg>
        </div>
        <div className="news-body">
          <div key={index} className="news-slide" style={{ background: `linear-gradient(115deg, ${slide.c1}, ${slide.c2} 72%)` }}>{slide.caption}</div>
        </div>
        <div className="news-bars">
          {SLIDES.map((s, k) => (
            <i key={s.caption} className={k === index ? "is-on" : ""} onClick={(e) => { e.stopPropagation(); setIndex(k); }} />
          ))}
        </div>
      </div>
    </div>
  );
}

export const DiscButtons = memo(function DiscButtons() {
  return (
    <svg className="disc-btns" viewBox="0 0 1920 1080">
      <defs>
        <linearGradient id="dcTop" gradientUnits="userSpaceOnUse" x1="700" y1="0" x2="1260" y2="0">
          <stop offset="0" stopColor="rgba(137,129,57,0.9)" />
          <stop offset="0.214" stopColor="rgba(159,145,54,0.9)" />
          <stop offset="0.464" stopColor="rgba(171,156,55,0.9)" />
          <stop offset="0.536" stopColor="rgba(165,150,51,0.9)" />
          <stop offset="0.75" stopColor="rgba(159,145,47,0.9)" />
          <stop offset="1" stopColor="rgba(143,136,51,0.9)" />
        </linearGradient>
        {DISC_BUTTONS.map((b, i) => (
          <clipPath id={`dcClip${i}`} key={b.action}><path d={b.path} /></clipPath>
        ))}
      </defs>
      <g transform="rotate(-1.817 968.8 918)">
        {DISC_BUTTONS.map((b, i) => (
          <g key={b.action} className="tab">
            <path d={b.path} className="tab-plate" transform="translate(5 6)" />
            <g clipPath={`url(#dcClip${i})`}>
              <rect x="640" y="840" width="660" height={b.band - 840} fill="url(#dcTop)" />
              <rect x="640" y={b.band} width="660" height={990 - b.band} fill="#FFE128" />
              <path d={b.path} className="tab-stroke" />
            </g>
            <path d={b.path} className="tab-edge" />
            <path d={b.path} className="tab-hit" data-action={b.action} aria-label={b.label} />
          </g>
        ))}
      </g>
      {DISC_BUTTONS.map((b) => (
        <g key={b.action} className="tab-label">
          <use href={`#${b.decal}`} className="decal" />
          <use href={`#${b.art}`} />
          <GlyphText text={b.label} g={b.text} />
        </g>
      ))}
    </svg>
  );
});

export function PlusMarks() {
  return (
    <>
      {PLUS_MARKS.map((mark, i) => (
        <div className="mark" data-decor={i} key={i}>
          <svg viewBox="-16 -16 32 32" aria-hidden="true">
            <g fill="#fff" fillOpacity={mark.a}>
              <rect x={mark.g[0]} y={-mark.wh / 2} width={mark.g[1]} height={mark.wh} />
              <rect x={-mark.g[0] - mark.g[1]} y={-mark.wh / 2} width={mark.g[1]} height={mark.wh} />
              <rect x={-mark.wv / 2} y={mark.g[2]} width={mark.wv} height={mark.g[3]} />
              <rect x={-mark.wv / 2} y={-mark.g[2] - mark.g[3]} width={mark.wv} height={mark.g[3]} />
            </g>
          </svg>
        </div>
      ))}
      <div className="yring" data-anchor="ring" style={{ width: (YELLOW_RING.r + 2) * 2, height: (YELLOW_RING.r + 2) * 2 }} />
    </>
  );
}
