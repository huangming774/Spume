import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

import {
  initialSquareCrop,
  minSquareCropSize,
  moveSquareCrop,
  renderAvatarFile,
  resizeSquareCrop,
  setSquareCropSize,
} from "../../lib/avatarCrop.js";

const HANDLES = ["nw", "ne", "sw", "se"];

const AvatarCropDialog = ({ file, busy = false, onCancel, onConfirm }) => {
  const stageRef = useRef(null);
  const imageRef = useRef(null);
  const dragRef = useRef(null);
  const confirmLock = useRef(false);
  const naturalRef = useRef({ w: 0, h: 0 });
  const [url, setUrl] = useState("");
  const [natural, setNatural] = useState({ w: 0, h: 0 });
  const [crop, setCrop] = useState(null);
  const [displayWidth, setDisplayWidth] = useState(0);
  const [error, setError] = useState("");

  useEffect(() => {
    const nextUrl = URL.createObjectURL(file);
    setUrl(nextUrl);
    setCrop(null);
    setError("");
    return () => URL.revokeObjectURL(nextUrl);
  }, [file]);

  useEffect(() => {
    naturalRef.current = natural;
  }, [natural]);

  useEffect(() => {
    if (!crop) return undefined;
    const onKey = (event) => {
      if (event.key === "Escape" && !busy) onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [busy, crop, onCancel]);

  useEffect(() => {
    const image = imageRef.current;
    if (!image) return undefined;
    const measure = () => setDisplayWidth(image.getBoundingClientRect().width);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(image);
    return () => observer.disconnect();
  }, [url, natural.w]);

  useEffect(() => {
    const stage = stageRef.current;
    if (!stage) return undefined;
    const onWheel = (event) => {
      if (busy) return;
      event.preventDefault();
      const { w, h } = naturalRef.current;
      const width = imageRef.current?.getBoundingClientRect().width || 0;
      const floor = minSquareCropSize(w, h, width);
      const factor = event.deltaY > 0 ? 1.08 : 0.92;
      setCrop((current) =>
        current ? setSquareCropSize(current, current.size * factor, w, h, floor) : current,
      );
    };
    stage.addEventListener("wheel", onWheel, { passive: false });
    return () => stage.removeEventListener("wheel", onWheel);
  }, [busy, url]);

  const minSize =
    natural.w > 0 ? minSquareCropSize(natural.w, natural.h, displayWidth) : 1;
  const scale = natural.w > 0 && displayWidth > 0 ? displayWidth / natural.w : 0;
  const box = crop && scale
    ? {
        left: crop.x * scale,
        top: crop.y * scale,
        size: crop.size * scale,
      }
    : null;
  const shortSide = Math.min(natural.w, natural.h);

  const onImageLoad = (event) => {
    const { naturalWidth, naturalHeight } = event.currentTarget;
    if (!naturalWidth || !naturalHeight) {
      setError("这张图片读不出来");
      return;
    }
    const next = { w: naturalWidth, h: naturalHeight };
    setNatural(next);
    setCrop(initialSquareCrop(naturalWidth, naturalHeight));
    setDisplayWidth(event.currentTarget.getBoundingClientRect().width);
  };

  const pointerDelta = (event) => {
    const drag = dragRef.current;
    const width = imageRef.current?.getBoundingClientRect().width || 0;
    const naturalWidth = naturalRef.current.w;
    if (!drag || !width || !naturalWidth) return null;
    const ratio = naturalWidth / width;
    return {
      drag,
      dx: (event.clientX - drag.x) * ratio,
      dy: (event.clientY - drag.y) * ratio,
      minSize: minSquareCropSize(naturalWidth, naturalRef.current.h, width),
    };
  };

  const onPointerDown = (event, mode) => {
    if (busy || !crop) return;
    event.preventDefault();
    event.stopPropagation();
    event.currentTarget.setPointerCapture(event.pointerId);
    dragRef.current = { mode, x: event.clientX, y: event.clientY, crop };
  };

  const onPointerMove = (event) => {
    const delta = pointerDelta(event);
    if (!delta) return;
    const { w, h } = naturalRef.current;
    if (delta.drag.mode === "move") {
      setCrop(moveSquareCrop(delta.drag.crop, delta.dx, delta.dy, w, h, delta.minSize));
      return;
    }
    setCrop(
      resizeSquareCrop(
        delta.drag.crop,
        delta.drag.mode,
        delta.dx,
        delta.dy,
        w,
        h,
        delta.minSize,
      ),
    );
  };

  const onPointerUp = () => {
    dragRef.current = null;
  };

  const confirm = async () => {
    if (confirmLock.current || !crop || !imageRef.current) return;
    confirmLock.current = true;
    try {
      const nextFile = await renderAvatarFile(imageRef.current, crop, file.type);
      await onConfirm(nextFile);
    } catch (cropError) {
      setError(cropError.message || "无法裁切这张图片");
    } finally {
      confirmLock.current = false;
    }
  };

  const rangeMin = Math.max(1, Math.round(minSize));
  const rangeMax = Math.max(rangeMin, Math.round(shortSide || rangeMin));

  return createPortal(
    <div className="owner-avatar-crop owner-console" role="dialog" aria-modal="true" aria-labelledby="owner-avatar-crop-title">
      <div className="owner-avatar-crop-panel owner-glass">
        <div className="owner-panel-title">
          <h2 id="owner-avatar-crop-title">裁切头像</h2>
        </div>
        <p className="owner-avatar-note">
          拖动方框选择区域，拖圆点或滚轮调整大小。保存的就是方框里的这一块。
        </p>
        <div className="owner-avatar-crop-layout">
          <div className="owner-avatar-crop-stage" ref={stageRef}>
            {url ? (
              <img
                ref={imageRef}
                src={url}
                alt=""
                draggable={false}
                onLoad={onImageLoad}
                onError={() => setError("这张图片读不出来")}
              />
            ) : null}
            {box ? (
              <div
                className="owner-avatar-crop-box"
                style={{ left: box.left, top: box.top, width: box.size, height: box.size }}
                onPointerDown={(event) => onPointerDown(event, "move")}
                onPointerMove={onPointerMove}
                onPointerUp={onPointerUp}
              >
                {HANDLES.map((handle) => (
                  <span
                    key={handle}
                    className={`owner-avatar-crop-handle owner-avatar-crop-handle--${handle}`}
                    onPointerDown={(event) => onPointerDown(event, handle)}
                    onPointerMove={onPointerMove}
                    onPointerUp={onPointerUp}
                  />
                ))}
              </div>
            ) : null}
          </div>
          {crop && url ? (
            <div className="owner-avatar-crop-side">
              <span>预览</span>
              <div
                className="owner-avatar-crop-preview"
                style={{
                  backgroundImage: `url(${url})`,
                  backgroundSize: `${(natural.w / crop.size) * 88}px ${(natural.h / crop.size) * 88}px`,
                  backgroundPosition: `${(-crop.x / crop.size) * 88}px ${(-crop.y / crop.size) * 88}px`,
                }}
              />
              <label className="owner-avatar-crop-range">
                裁切大小
                <input
                  type="range"
                  min={rangeMin}
                  max={rangeMax}
                  value={Math.min(rangeMax, Math.max(rangeMin, Math.round(crop.size)))}
                  disabled={busy}
                  onChange={(event) => {
                    const nextSize = Number(event.target.value);
                    setCrop((current) =>
                      current
                        ? setSquareCropSize(current, nextSize, natural.w, natural.h, minSize)
                        : current,
                    );
                  }}
                />
              </label>
            </div>
          ) : null}
        </div>
        {error ? <p className="owner-alert">{error}</p> : null}
        <div className="owner-avatar-crop-actions">
          <button type="button" className="owner-secondary" onClick={onCancel} disabled={busy}>
            取消
          </button>
          <button type="button" className="owner-primary" onClick={confirm} disabled={busy || !crop}>
            {busy ? "上传中..." : "使用这张"}
          </button>
        </div>
      </div>
    </div>,
    document.body,
  );
};

export default AvatarCropDialog;
