import { useEffect, useRef } from 'react';
import { useWeather } from './WeatherContext';

type Drop = {
  x: number;
  y: number;
  len: number;
  speed: number;
  opacity: number;
  width: number;
};

export function RainCanvas() {
  const { rainy } = useWeather();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const runningRef = useRef(false);
  const dropsRef = useRef<Drop[]>([]);
  const animRef = useRef<number | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (!rainy || reduced) {
      runningRef.current = false;
      if (animRef.current) cancelAnimationFrame(animRef.current);
      const ctx = canvas.getContext('2d');
      if (ctx) ctx.clearRect(0, 0, canvas.width, canvas.height);
      return;
    }

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let w = 0;
    let h = 0;

    const resize = () => {
      w = canvas.width = window.innerWidth;
      h = canvas.height = window.innerHeight;
      const density = Math.min(280, Math.floor((w * h) / 5500));
      dropsRef.current = Array.from({ length: density }, () => ({
        x: Math.random() * w,
        y: Math.random() * h,
        len: 8 + Math.random() * 22,
        speed: 14 + Math.random() * 18,
        opacity: 0.15 + Math.random() * 0.45,
        width: 0.6 + Math.random() * 1.4,
      }));
    };

    const draw = () => {
      if (!runningRef.current) return;
      ctx.clearRect(0, 0, w, h);
      const windX = 2.2;
      for (const d of dropsRef.current) {
        const x2 = d.x + windX * (d.len / 8);
        const y2 = d.y + d.len;
        const grad = ctx.createLinearGradient(d.x, d.y, x2, y2);
        grad.addColorStop(0, 'rgba(160, 190, 220, 0)');
        grad.addColorStop(0.15, `rgba(180, 210, 235, ${d.opacity * 0.6})`);
        grad.addColorStop(0.5, `rgba(210, 228, 245, ${d.opacity})`);
        grad.addColorStop(1, `rgba(140, 175, 210, ${d.opacity * 0.3})`);
        ctx.beginPath();
        ctx.strokeStyle = grad;
        ctx.lineWidth = d.width;
        ctx.lineCap = 'round';
        ctx.moveTo(d.x, d.y);
        ctx.lineTo(x2, y2);
        ctx.stroke();
        d.y += d.speed;
        d.x += windX;
        if (d.y > h + 30) {
          d.y = -d.len - Math.random() * 40;
          d.x = Math.random() * w;
        }
        if (d.x > w + 20) d.x = -20;
      }
      animRef.current = requestAnimationFrame(draw);
    };

    resize();
    runningRef.current = true;
    draw();

    const onResize = () => {
      if (runningRef.current) resize();
    };
    window.addEventListener('resize', onResize);

    return () => {
      runningRef.current = false;
      if (animRef.current) cancelAnimationFrame(animRef.current);
      window.removeEventListener('resize', onResize);
    };
  }, [rainy]);

  return <canvas ref={canvasRef} id="rainCanvas" className="rain-canvas" aria-hidden="true" />;
}
