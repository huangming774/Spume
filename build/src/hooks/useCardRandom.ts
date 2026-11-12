import { useEffect } from 'react';

const SKINS = [
  'moment-skin--aurora',
  'moment-skin--sakura',
  'moment-skin--ticket',
  'moment-skin--envelope',
  'moment-skin--watercolor',
  'moment-skin--starry',
  'moment-skin--candy',
  'moment-skin--journal',
  'moment-skin--wave',
  'moment-skin--frost',
  'moment-skin--sunset',
  'moment-skin--mint',
  'moment-skin--ribbon',
  'moment-skin--neon',
  'moment-skin--glass',
  'moment-skin--stamp',
];

const CALM_SKINS = [
  'moment-skin--aurora',
  'moment-skin--sakura',
  'moment-skin--watercolor',
  'moment-skin--starry',
  'moment-skin--candy',
  'moment-skin--frost',
  'moment-skin--sunset',
  'moment-skin--mint',
  'moment-skin--ribbon',
  'moment-skin--neon',
  'moment-skin--glass',
  'moment-skin--stamp',
];

const POETIC = 'moment-skin--mist';

const FEEDS: { root: string; tilt: boolean; skins: string[] }[] = [
  { root: '.gallery-feed', tilt: true, skins: SKINS },
  { root: '.archive-timeline', tilt: false, skins: CALM_SKINS },
  { root: '.links-stack', tilt: false, skins: CALM_SKINS },
];

function shuffle<T>(arr: T[]): T[] {
  const a = arr.slice();
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

function applyFeed(cfg: (typeof FEEDS)[0]) {
  const feed = document.querySelector(cfg.root);
  if (!feed) return;
  const cards = feed.querySelectorAll<HTMLElement>('.moment');
  if (!cards.length) return;

  const pool = shuffle(cfg.skins);
  let pi = 0;

  cards.forEach((el) => {
    [...SKINS, POETIC].forEach((c) => el.classList.remove(c));
    el.style.setProperty(
      '--moment-tilt',
      cfg.tilt ? `${(Math.random() * 5 - 2.5).toFixed(2)}deg` : '0deg',
    );

    if (el.classList.contains('moment--poetic')) {
      el.classList.add(POETIC);
      el.style.setProperty('--moment-tilt', '0deg');
      return;
    }

    el.classList.add(pool[pi % pool.length]);
    pi += 1;
  });
}

export function useCardRandom(deps: unknown[] = []) {
  useEffect(() => {
    FEEDS.forEach(applyFeed);
  }, deps);
}

