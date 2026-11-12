/** @param {"bangumi"|"radar"} variant */
export const makePosterDataUri = (label, variant = "bangumi") => {
  const safe = String(label).replace(/[<>&"]/g, "").slice(0, 20);
  if (variant === "radar") {
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="640" height="360" viewBox="0 0 640 360">
      <defs>
        <linearGradient id="sky" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0%" stop-color="#FFF9E6"/>
          <stop offset="45%" stop-color="#FFEAF4"/>
          <stop offset="100%" stop-color="#EAF6FF"/>
        </linearGradient>
        <pattern id="grid" width="24" height="24" patternUnits="userSpaceOnUse">
          <path d="M 24 0 L 0 0 0 24" fill="none" stroke="#7C5CFF" stroke-width="0.6" opacity="0.12"/>
        </pattern>
        <filter id="grain">
          <feTurbulence type="fractalNoise" baseFrequency="0.95" numOctaves="3"/>
          <feColorMatrix type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 0.08 0"/>
        </filter>
      </defs>
      <rect width="640" height="360" fill="url(#sky)"/>
      <rect width="640" height="360" fill="url(#grid)"/>
      <rect width="640" height="360" filter="url(#grain)" opacity="0.5"/>
      <circle cx="88" cy="62" r="3" fill="#FF6BAA" opacity="0.55"/>
      <circle cx="520" cy="48" r="2.5" fill="#00C2FF" opacity="0.6"/>
      <circle cx="560" cy="120" r="2" fill="#7C5CFF" opacity="0.45"/>
      <path d="M120 80l4 8 9 1-6.5 6.5 1.5 9-8-4.5-8 4.5 1.5-9-6.5-6.5 9-1z" fill="#FFD166" opacity="0.7"/>
      <path d="M480 200l3 6 7 .8-5 5 1.2 7-6.2-3.5-6.2 3.5 1.2-7-5-5 7-.8z" fill="#FF6BAA" opacity="0.55"/>
      <text x="320" y="188" text-anchor="middle" font-family="system-ui,sans-serif" font-size="11" letter-spacing="0.25em" fill="#7C5CFF" opacity="0.35">CREATOR FEED</text>
    </svg>`;
    return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
  }

  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="600" height="800" viewBox="0 0 600 800">
    <defs>
      <linearGradient id="g" x1="0" y1="0" x2="0.5" y2="1">
        <stop offset="0%" stop-color="#EAF6FF"/>
        <stop offset="50%" stop-color="#FFEAF4"/>
        <stop offset="100%" stop-color="#F3E8FF"/>
      </linearGradient>
    </defs>
    <rect width="600" height="800" fill="url(#g)"/>
    <circle cx="120" cy="140" r="40" fill="#00C2FF" opacity="0.12"/>
    <circle cx="480" cy="620" r="70" fill="#FF6BAA" opacity="0.1"/>
    <text x="300" y="420" text-anchor="middle" font-family="system-ui,sans-serif" font-size="12" fill="#7C5CFF" opacity="0.4">${safe || "ANIME"}</text>
  </svg>`;
  return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
};

export const hasRemoteCover = (item) =>
  Boolean(item?.coverUrl && !String(item.coverUrl).startsWith("data:"));

export const resolveCoverSrc = (item, apiBase) => {
  if (!item?.coverUrl) return null;
  if (item.coverUrl.startsWith("http") || item.coverUrl.startsWith("data:")) {
    return item.coverUrl;
  }
  const base = (apiBase || "").replace(/\/$/, "");
  return base ? `${base}${item.coverUrl}` : item.coverUrl;
};
