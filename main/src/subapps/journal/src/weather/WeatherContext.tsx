import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { useI18n } from '../i18n/I18nContext';

const STORAGE_KEY = 'siteWeather';

type WeatherContextValue = {
  rainy: boolean;
  toggleWeather: () => void;
  weatherLabel: string;
  weatherIcon: string;
};

const WeatherContext = createContext<WeatherContextValue | null>(null);

function readRainy(): boolean {
  try {
    return localStorage.getItem(STORAGE_KEY) === 'rainy';
  } catch {
    return false;
  }
}

export function WeatherProvider({ children }: { children: ReactNode }) {
  const { lang } = useI18n();
  const [rainy, setRainy] = useState(readRainy);

  const toggleWeather = useCallback(() => {
    setRainy((prev) => {
      const next = !prev;
      try {
        localStorage.setItem(STORAGE_KEY, next ? 'rainy' : 'sunny');
      } catch {
        /* ignore */
      }
      return next;
    });
  }, []);

  useEffect(() => {
    document.documentElement.classList.toggle('weather-rainy', rainy);
    document.body.classList.toggle('weather-rainy', rainy);
  }, [rainy]);

  const weatherLabel = rainy
    ? lang === 'en'
      ? 'Sunny'
      : '晴天'
    : lang === 'en'
      ? 'Rainy'
      : '雨天';
  const weatherIcon = rainy ? '☀' : '☔';

  const value = useMemo(
    () => ({ rainy, toggleWeather, weatherLabel, weatherIcon }),
    [rainy, toggleWeather, weatherLabel, weatherIcon],
  );

  return <WeatherContext.Provider value={value}>{children}</WeatherContext.Provider>;
}

export function useWeather() {
  const ctx = useContext(WeatherContext);
  if (!ctx) throw new Error('useWeather must be used within WeatherProvider');
  return ctx;
}
