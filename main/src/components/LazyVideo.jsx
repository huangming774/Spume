import { useEffect, useRef, useState } from "react";

/**
 * Loads video only when near viewport; plays only while visible.
 * Reduces initial bandwidth vs autoplay on all feature-*.mp4 at once.
 */
const LazyVideo = ({ src, className = "", priority = false }) => {
  const wrapRef = useRef(null);
  const videoRef = useRef(null);
  const [active, setActive] = useState(priority);

  useEffect(() => {
    if (priority) return;
    const node = wrapRef.current;
    if (!node) return;

    const io = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setActive(true);
          io.disconnect();
        }
      },
      { rootMargin: "280px 0px", threshold: 0.01 }
    );
    io.observe(node);
    return () => io.disconnect();
  }, [priority]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video || !active) return;

    const playIo = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          video.play().catch(() => {});
        } else {
          video.pause();
        }
      },
      { threshold: 0.15 }
    );
    playIo.observe(wrapRef.current);
    return () => playIo.disconnect();
  }, [active]);

  return (
    <div ref={wrapRef} className={`relative size-full ${className}`}>
      {active ? (
        <video
          ref={videoRef}
          src={src}
          loop
          muted
          playsInline
          preload="metadata"
          className="absolute left-0 top-0 size-full object-cover object-center"
        />
      ) : (
        <div
          className="absolute inset-0 bg-gradient-to-br from-neutral-800 to-neutral-900"
          aria-hidden
        />
      )}
    </div>
  );
};

export default LazyVideo;
