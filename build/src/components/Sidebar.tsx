import { useState } from 'react';
import { NavLink } from 'react-router-dom';
import { useI18n } from '../i18n/I18nContext';
import { useWeather } from '../weather/WeatherContext';

const AVATAR = 'https://tzyy-1330068502.cos.ap-beijing.myqcloud.com/1.png';

const NAV = [
  { to: '/', icon: '🏠', key: 'navHome' as const },
  { to: '/archives', icon: '📂', key: 'navArchive' as const },
  { to: '/articles', icon: '📝', key: 'navArticle' as const },
];

export function Sidebar() {
  const { t, lang, toggleLang } = useI18n();
  const { toggleWeather, weatherLabel, weatherIcon, rainy } = useWeather();
  const [collapsed, setCollapsed] = useState(false);

  return (
    <aside className={`sidebar${collapsed ? ' collapsed' : ''}`}>
      <button
        type="button"
        className="sidebar-toggle"
        onClick={() => setCollapsed((c) => !c)}
        title="折叠导航栏"
      >
        ☰
      </button>
      <div className="sidebar-header">
        <div className="avatar">
          <img src={AVATAR} alt="" />
        </div>
        <div>
          <div className="blog-name blog-name-full">{t('blogTitle')}</div>
        </div>
      </div>
      <div className="sidebar-social">
        <a href="https://space.bilibili.com/1061280173" target="_blank" rel="noopener noreferrer" title="B站">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <rect x="3.5" y="5" width="17" height="12.5" rx="2" stroke="#aaa" strokeWidth="1.2" />
            <path d="M6 2l2 3M18 2l-2 3" stroke="#aaa" strokeWidth="1.2" strokeLinecap="round" />
            <path d="M8 9v5M11 9v5M14 9v5M16 9v5" stroke="#aaa" strokeWidth="1" strokeLinecap="round" />
          </svg>
        </a>
        <a href="https://github.com/bistutzyy" target="_blank" rel="noopener noreferrer" title="GitHub">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" aria-hidden="true">
            <path
              d="M12 2C6.48 2 2 6.48 2 12c0 4.42 2.87 8.17 6.84 9.5.5.08.66-.23.66-.5v-1.69c-2.78.6-3.36-1.34-3.36-1.34-.46-1.16-1.11-1.47-1.11-1.47-.91-.62.07-.6.07-.6 1 .07 1.53 1.03 1.53 1.03.9 1.52 2.34 1.08 2.91.83.1-.65.35-1.09.63-1.34-2.22-.25-4.55-1.11-4.55-4.92 0-1.09.39-1.98 1.03-2.68-.1-.25-.45-1.27.1-2.65 0 0 .84-.27 2.75 1.02A9.6 9.6 0 0112 6.8c.85.004 1.7.11 2.5.34 1.91-1.29 2.75-1.02 2.75-1.02.55 1.38.2 2.4.1 2.65.64.7 1.03 1.59 1.03 2.68 0 3.82-2.34 4.66-4.57 4.91.36.31.68.92.68 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12c0-5.52-4.48-10-10-10z"
              fill="#aaa"
            />
          </svg>
        </a>
      </div>
      <nav className="sidebar-nav">
        {NAV.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === '/'}
            className={({ isActive }) => `nav-item${isActive ? ' active' : ''}`}
          >
            <span className="nav-icon">{item.icon}</span>
            <span className="nav-label">{t(item.key)}</span>
          </NavLink>
        ))}
      </nav>
      <div className="lang-toggle-wrap">
        <button type="button" className="lang-toggle" onClick={toggleLang}>
          {lang === 'zh' ? 'English' : '中文'}
        </button>
        <button
          type="button"
          className="weather-toggle"
          onClick={toggleWeather}
          aria-pressed={rainy}
        >
          <span className="weather-toggle-icon" aria-hidden="true">
            {weatherIcon}
          </span>
          <span className="weather-toggle-label">{weatherLabel}</span>
        </button>
      </div>
      <div className="sidebar-footer">{t('footerLine')}</div>
    </aside>
  );
}
