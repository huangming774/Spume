import { useEffect, useState } from "react";
import { createPortal } from "react-dom";
import { App as EndfieldMenu } from "../../../endfield_menu/src/App.tsx";
import "virtual:where-menu.css";

const WhereOverlay = ({ open, warm, onClose }) => {
  const [mounted, setMounted] = useState(open || warm);

  useEffect(() => {
    if (open || warm) {
      setMounted(true);
      return undefined;
    }
    const id = window.setTimeout(() => setMounted(true), 400);
    return () => window.clearTimeout(id);
  }, [open, warm]);

  useEffect(() => {
    if (!open) return undefined;
    const previous = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => {
      document.body.style.overflow = previous;
    };
  }, [open]);

  if (!mounted) return null;

  return createPortal(
    <div
      className="where-menu"
      data-shown={open ? "1" : "0"}
      role="dialog"
      aria-modal={open}
      aria-hidden={!open}
      aria-label="你在哪里？"
    >
      <EndfieldMenu
        quick
        active={open}
        onPhase={(phase) => {
          if (phase === "closed") onClose();
        }}
      />
    </div>,
    document.body,
  );
};

export default WhereOverlay;
