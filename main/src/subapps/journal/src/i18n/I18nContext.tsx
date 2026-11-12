import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { STR, type I18nKey, type Lang } from './strings';

type I18nContextValue = {
  lang: Lang;
  t: (key: I18nKey) => string;
  toggleLang: () => void;
};

const I18nContext = createContext<I18nContextValue | null>(null);

function readLang(): Lang {
  try {
    return localStorage.getItem('siteLang') === 'en' ? 'en' : 'zh';
  } catch {
    return 'zh';
  }
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLang] = useState<Lang>(readLang);

  const t = useCallback(
    (key: I18nKey) => STR[lang][key] ?? key,
    [lang],
  );

  const toggleLang = useCallback(() => {
    setLang((prev) => {
      const next: Lang = prev === 'zh' ? 'en' : 'zh';
      try {
        localStorage.setItem('siteLang', next);
      } catch {
        /* ignore */
      }
      return next;
    });
  }, []);

  useEffect(() => {
    document.documentElement.lang = lang === 'en' ? 'en' : 'zh-CN';
  }, [lang]);

  const value = useMemo(() => ({ lang, t, toggleLang }), [lang, t, toggleLang]);

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n() {
  const ctx = useContext(I18nContext);
  if (!ctx) throw new Error('useI18n must be used within I18nProvider');
  return ctx;
}
