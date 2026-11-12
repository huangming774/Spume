/** Lightweight mock payloads — used when API is offline (zero extra network). */

export const mockBangumiList = [
  {
    id: "b1",
    title: "A Certain Scientific Railgun",
    watched: 12,
    total: 24,
    latestEpisode: 24,
    coverUrl: null,
    hue: 200,
  },
  {
    id: "b2",
    title: "Healing Sketchbook",
    watched: 8,
    total: 12,
    latestEpisode: 10,
    coverUrl: null,
    hue: 280,
  },
  {
    id: "b3",
    title: "Chronicles of Creation",
    watched: 3,
    total: 13,
    latestEpisode: 7,
    coverUrl: null,
    hue: 140,
  },
  {
    id: "b4",
    title: "Reimu Garden",
    watched: 6,
    total: 12,
    latestEpisode: 9,
    coverUrl: null,
    hue: 35,
  },
];

export const mockRadarList = [
  {
    id: "r1",
    creatorName: "UP · TechLullaby",
    latestText:
      "Posted a calm devlog with a fresh cover — worth a peek tonight.",
    isNew: true,
    linkUrl: "https://space.bilibili.com/",
  },
  {
    id: "r2",
    creatorName: "UP · SystemsGarden",
    latestText: "New video: stream caches, latency tuning, no ads.",
    isNew: false,
    linkUrl: "https://space.bilibili.com/",
  },
  {
    id: "r3",
    creatorName: "UP · MythicCoder",
    latestText: "Radar sync: 3 creators added to the watch list.",
    isNew: true,
    linkUrl: "https://space.bilibili.com/",
  },
  {
    id: "r4",
    creatorName: "UP · CalmCanvas",
    latestText: "Mobile grid polish for ACG navigation cards.",
    isNew: false,
    linkUrl: "https://space.bilibili.com/",
  },
];

export const mockGuestbookEntries = [
  {
    id: 1,
    name: "visitor",
    content: "Clean grid. The progress bars feel right on mobile.",
    createdAt: "2026-06-02T10:00:00Z",
  },
  {
    id: 2,
    name: "geek",
    content: "No ads, just signal. Keep the radar pulsing.",
    createdAt: "2026-06-02T11:30:00Z",
  },
];
