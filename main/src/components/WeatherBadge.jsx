import { useEffect, useState } from "react";
import {
  LuCloud,
  LuCloudDrizzle,
  LuCloudFog,
  LuCloudHail,
  LuCloudLightning,
  LuCloudMoon,
  LuCloudRain,
  LuCloudRainWind,
  LuCloudSnow,
  LuCloudSun,
  LuCloudy,
  LuMoon,
  LuSun,
} from "react-icons/lu";

import { fetchWeather } from "../services/weatherApi.js";

const REFRESH_MS = 10 * 60 * 1000;

// WMO 天气代码 → 图标。分组取自 Open-Meteo 文档的标准代码表。
const iconFor = (code, isDay) => {
  if (code === 0) return isDay ? LuSun : LuMoon;
  if (code === 1 || code === 2) return isDay ? LuCloudSun : LuCloudMoon;
  if (code === 3) return LuCloudy;
  if (code === 45 || code === 48) return LuCloudFog;
  if (code === 51 || code === 53 || code === 55 || code === 56 || code === 57)
    return LuCloudDrizzle;
  if (code === 61 || code === 63 || code === 65 || code === 66 || code === 67)
    return LuCloudRain;
  if (code === 71 || code === 73 || code === 75 || code === 77 || code === 85 || code === 86)
    return LuCloudSnow;
  if (code === 80 || code === 81 || code === 82) return LuCloudRainWind;
  if (code === 95) return LuCloudLightning;
  if (code === 96 || code === 99) return LuCloudHail;
  return LuCloud;
};

const WeatherBadge = ({ className = "" }) => {
  const [weather, setWeather] = useState(null);

  useEffect(() => {
    let cancelled = false;
    let refreshTimer = null;
    let request = null;

    const load = async () => {
      request?.abort();
      const controller = new AbortController();
      request = controller;
      const timeout = window.setTimeout(() => controller.abort(), 5000);
      try {
        const next = await fetchWeather({ signal: controller.signal });
        if (!cancelled) setWeather(next);
      } catch {
        // 天气超时或失败只留占位，不占用页面其它请求。
      } finally {
        window.clearTimeout(timeout);
      }
    };

    load();
    refreshTimer = window.setInterval(load, REFRESH_MS);
    return () => {
      cancelled = true;
      request?.abort();
      if (refreshTimer) window.clearInterval(refreshTimer);
    };
  }, []);

  const { city, region, temperature, weatherCode, weatherDescription, isDay } = weather || {};
  const Icon = weather ? iconFor(weatherCode, isDay !== false) : LuCloud;
  const tempText =
    weather && typeof temperature === "number" ? `${Math.round(temperature)}°` : "--°";
  const locationText = city || region;
  const title = weather
    ? [locationText, weatherDescription, tempText].filter(Boolean).join(" · ")
    : "天气加载中";

  return (
    <span
      className={`weather-badge ${className}`}
      title={title}
      aria-label={title}
    >
      <span className="weather-badge-chip">
        <Icon aria-hidden="true" className="weather-badge-icon" />
        <span>{tempText}</span>
      </span>
    </span>
  );
};

export default WeatherBadge;
