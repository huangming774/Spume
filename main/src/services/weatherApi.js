export async function fetchWeather({ fetchImpl = fetch, signal } = {}) {
  const res = await fetchImpl("/api/weather", {
    headers: { Accept: "application/json" },
    cache: "no-store",
    signal,
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(data.message || data.error || `HTTP ${res.status}`);
  }
  return {
    city: data.city || "",
    region: data.region || "",
    temperature: data.temperature,
    weatherCode: data.weatherCode,
    weatherDescription: data.weatherDescription || "",
    windSpeed: data.windSpeed,
    isDay: data.isDay,
  };
}
