import { useEffect, useState } from "react";
import { WELL_R } from "../scene/meshes";

const ZOOM = 12;
const TILE = 256;
const SIZE = WELL_R * 2;
const LAYERS = ["vec_w", "cva_w"] as const;

type Place = { lat: number; lon: number; label: string };

type Tile = { key: string; src: string; left: number; top: number };

/** 天地图浏览器端 tk，来自 main/.env.local 的 VITE_TIANDITU_TK。 */
const TIANDITU_TK = (import.meta.env.VITE_TIANDITU_TK || "").trim();

function finite(value: unknown): number | null {
  const n = typeof value === "number" ? value : Number(value);
  return Number.isFinite(n) ? n : null;
}

function labelOf(city: unknown, region: unknown): string {
  const c = typeof city === "string" ? city.trim() : "";
  const r = typeof region === "string" ? region.trim() : "";
  if (c && r && c !== r) return `${r} ${c}`;
  return c || r;
}

async function readPlace(url: string): Promise<Place | null> {
  const res = await fetch(url, { headers: { Accept: "application/json" }, cache: "no-store" });
  if (!res.ok) return null;
  const data = await res.json();
  if (data?.located === false) return null;
  const lat = finite(data?.lat ?? data?.latitude);
  const lon = finite(data?.lon ?? data?.longitude);
  if (lat === null || lon === null) return null;
  return { lat, lon, label: labelOf(data?.city, data?.region ?? data?.regionName) };
}

async function locate(): Promise<Place> {
  try {
    const place = await readPlace("/api/client/place");
    if (place) return place;
  } catch {
    // 接口没起来时用上海兜底，避免圆框空着。
  }
  return { lat: 31.2304, lon: 121.4737, label: "上海" };
}

function tilesFor(lat: number, lon: number): Tile[] {
  const n = 2 ** ZOOM;
  const worldX = ((lon + 180) / 360) * n * TILE;
  const sine = Math.min(Math.max(Math.sin((lat * Math.PI) / 180), -0.9999), 0.9999);
  const worldY = (0.5 - Math.log((1 + sine) / (1 - sine)) / (4 * Math.PI)) * n * TILE;
  const originX = worldX - SIZE / 2;
  const originY = worldY - SIZE / 2;
  const x0 = Math.floor(originX / TILE);
  const y0 = Math.floor(originY / TILE);
  const x1 = Math.floor((originX + SIZE - 1) / TILE);
  const y1 = Math.floor((originY + SIZE - 1) / TILE);
  const tiles: Tile[] = [];
  for (let ty = y0; ty <= y1; ty += 1) {
    if (ty < 0 || ty >= n) continue;
    for (let tx = x0; tx <= x1; tx += 1) {
      const wrapped = ((tx % n) + n) % n;
      const left = tx * TILE - originX;
      const top = ty * TILE - originY;
      const shard = Math.abs(wrapped + ty) % 8;
      for (const layer of LAYERS) {
        tiles.push({
          key: `${layer}-${wrapped}-${ty}`,
          left,
          top,
          src: `https://t${shard}.tianditu.gov.cn/DataServer?T=${layer}&x=${wrapped}&y=${ty}&l=${ZOOM}&tk=${TIANDITU_TK}`,
        });
      }
    }
  }
  return tiles;
}

export function WellMap() {
  const [place, setPlace] = useState<Place | null>(null);

  useEffect(() => {
    let cancel = false;
    locate().then((next) => {
      if (!cancel) setPlace(next);
    });
    return () => {
      cancel = true;
    };
  }, []);

  const tiles = place ? tilesFor(place.lat, place.lon) : [];

  return (
    <div className="well-map" data-anchor="well" aria-hidden="true">
      <div className="well-tiles">
        {tiles.map((tile) => (
          <img
            key={tile.key}
            src={tile.src}
            alt=""
            draggable={false}
            style={{ left: tile.left, top: tile.top }}
            onError={(event) => {
              event.currentTarget.style.visibility = "hidden";
            }}
          />
        ))}
      </div>
      {place ? (
        <>
          <i className="well-mark" />
          {place.label ? <span className="well-name">{place.label}</span> : null}
        </>
      ) : null}
    </div>
  );
}
