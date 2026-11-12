import { useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import cosmosCss from "../../../previews/project-gallery/src/cosmos.css?inline";
import themeCss from "../../../previews/project-gallery/src/gallery-themes.css?inline";
import readerCss from "../../../previews/project-gallery/src/reader.css?inline";
import styleCss from "../../../previews/project-gallery/src/style.css?inline";
import { cosmosMarkup } from "./cosmosMarkup.js";

const cosmosStyles = [styleCss, cosmosCss, themeCss, readerCss].join("\n");

function isSiteHome(anchor) {
  const raw = anchor.getAttribute("href") || "";
  if (raw === "/" || raw === "https://taozhiyy.top" || raw === "https://taozhiyy.top/") return true;
  try {
    const url = new URL(anchor.href, window.location.origin);
    return url.pathname === "/" && (url.hostname === "taozhiyy.top" || url.origin === window.location.origin);
  } catch {
    return false;
  }
}

export default function CosmosPage() {
  const hostRef = useRef(null);
  const navigate = useNavigate();

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return undefined;
    const style = document.createElement("style");
    style.dataset.embeddedSection = "cosmos";
    style.textContent = cosmosStyles;
    document.head.append(style);
    host.innerHTML = cosmosMarkup;
    const previousOverflow = document.body.style.overflow;
    const previousTitle = document.title;
    document.body.style.overflow = "hidden";
    document.title = "奇想馆 · 项目星空";
    let stopped = false;
    let dispose = () => {};
    const onClick = (event) => {
      const anchor = event.target.closest?.("a");
      if (!anchor || !host.contains(anchor) || !isSiteHome(anchor)) return;
      event.preventDefault();
      navigate("/");
    };
    host.addEventListener("click", onClick);
    import("../../../previews/project-gallery/src/main.js").then(({ mountProjectGallery }) => {
      if (stopped) return;
      const stop = mountProjectGallery(host);
      if (stopped) stop();
      else dispose = stop;
    });
    return () => {
      stopped = true;
      dispose();
      host.removeEventListener("click", onClick);
      host.innerHTML = "";
      style.remove();
      document.body.style.overflow = previousOverflow;
      document.title = previousTitle;
    };
  }, [navigate]);

  return <div ref={hostRef} />;
}
