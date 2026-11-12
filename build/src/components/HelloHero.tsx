import { useEffect, useRef } from 'react';
import opentype from 'opentype.js';

const NS = 'http://www.w3.org/2000/svg';
const FONT_SIZE = 130;

function textToPaths(font: opentype.Font, t: string, fs: number) {
  const sc = fs / font.unitsPerEm;
  const paths: { d: string; x: number }[] = [];
  let x = 0;
  let ascender = 0;
  let descender = 0;
  for (const c of [...t]) {
    const g = font.charToGlyph(c);
    const path = g.getPath(x, 0, fs);
    paths.push({ d: path.toPathData(2), x });
    x += (g.advanceWidth ?? 0) * sc;
    ascender = Math.max(ascender, (font.ascender ?? 0) * sc);
    descender = Math.max(descender, Math.abs((font.descender ?? 0) * sc));
  }
  return { paths, totalWidth: x, ascender, descender };
}

export function HelloHero() {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    let cancelled = false;

    (async () => {
      let b64 = '';
      try {
        const res = await fetch(new URL('../assets/hello-font.b64', import.meta.url));
        if (res.ok) b64 = (await res.text()).trim();
      } catch {
        /* no font file */
      }

      if (!b64 || cancelled) {
        el.innerHTML =
          '<div class="hero-fallback" style="font-size:clamp(64px,12vw,130px);font-weight:300;letter-spacing:0.06em;color:#6a7a9a;opacity:0.85">Hello</div>';
        return;
      }

      const bin = atob(b64);
      const bytes = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
      const font = opentype.parse(bytes.buffer);
      const r = textToPaths(font, 'Hello', FONT_SIZE);
      const svg = document.createElementNS(NS, 'svg');
      const pad = FONT_SIZE * 0.35;
      svg.setAttribute('viewBox', `0 0 ${r.totalWidth + pad * 2} ${r.ascender + r.descender + pad * 2}`);
      svg.setAttribute('class', 'hero-svg');

      const defs = document.createElementNS(NS, 'defs');
      [0, 1].forEach((gi) => {
        const grad = document.createElementNS(NS, 'linearGradient');
        grad.id = `g${gi}`;
        grad.setAttribute('x1', '0%');
        grad.setAttribute('y1', '0%');
        grad.setAttribute('x2', '100%');
        grad.setAttribute('y2', '10%');
        ['#7b93db', '#9b8fd4', '#c880c0'].forEach((color, i) => {
          const stop = document.createElementNS(NS, 'stop');
          stop.setAttribute('offset', `${(i / 2) * 100}%`);
          stop.setAttribute('stop-color', color);
          grad.appendChild(stop);
        });
        defs.appendChild(grad);
      });
      svg.appendChild(defs);

      r.paths.forEach((p, i) => {
        const tmp = document.createElementNS(NS, 'svg');
        const tmpPath = document.createElementNS(NS, 'path') as SVGPathElement;
        tmpPath.setAttribute('d', p.d);
        tmp.appendChild(tmpPath);
        tmp.style.cssText = 'position:absolute;visibility:hidden;width:0;height:0';
        document.body.appendChild(tmp);
        const len = tmpPath.getTotalLength();
        tmp.remove();

        const path = document.createElementNS(NS, 'path') as SVGPathElement;
        path.setAttribute('d', p.d);
        path.classList.add('stroke-path', 'fill-after');
        path.style.setProperty('--len', `${len}`);
        path.style.setProperty('--dur', '1.5s');
        path.style.setProperty('--delay', `${i * 0.12}s`);
        path.style.strokeDasharray = `${len}`;
        path.style.strokeDashoffset = `${len}`;
        path.style.stroke = 'url(#g0)';
        path.style.strokeWidth = '2';
        path.style.fill = 'url(#g1)';
        path.style.fillOpacity = '0';
        svg.appendChild(path);
      });
      if (!cancelled) {
        el.innerHTML = '';
        el.appendChild(svg);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  return <div className="hero" id="hero" ref={ref} />;
}
