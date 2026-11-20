const osIcon = (os = "") => {
  if (/Windows/i.test(os)) return "🪟";
  if (/Android/i.test(os)) return "🤖";
  if (/iOS|iPhone|iPad/i.test(os)) return "";
  if (/macOS|Mac/i.test(os)) return "🍎";
  if (/Linux/i.test(os)) return "🐧";
  if (/ChromeOS/i.test(os)) return "💻";
  return "💻";
};

const browserIcon = (b = "") => {
  if (/Edge/i.test(b)) return "🌊";
  if (/Firefox/i.test(b)) return "🦊";
  if (/Chrome/i.test(b)) return "🌐";
  if (/Safari/i.test(b)) return "🧭";
  if (/Opera/i.test(b)) return "🎭";
  return "🌐";
};

/**
 * 留言客户端信息图标行：操作系统 / 浏览器 / 地区公开显示；
 * 掩码 IP 仅站长（item.ipMasked 由后端只对站长下发）可见。
 */
const ClientMeta = ({ item, className = "" }) => {
  if (!item) return null;
  const has = item.uaOs || item.uaBrowser || item.ipRegion || item.ipMasked;
  if (!has) return null;
  return (
    <p className={`flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs text-[#9AA4B2] ${className}`}>
      {item.uaOs && (
        <span className="inline-flex items-center gap-1" title={item.uaOs}>
          <span aria-hidden>{osIcon(item.uaOs)}</span>
          {item.uaOs}
        </span>
      )}
      {item.uaBrowser && (
        <span className="inline-flex items-center gap-1" title={item.uaBrowser}>
          <span aria-hidden>{browserIcon(item.uaBrowser)}</span>
          {item.uaBrowser}
        </span>
      )}
      {item.ipRegion && (
        <span className="inline-flex items-center gap-1" title={item.ipRegion}>
          <span aria-hidden>🌏</span>
          {item.ipRegion}
        </span>
      )}
      {item.ipMasked && (
        <span className="inline-flex items-center gap-1" title="掩码 IP（仅站长可见）">
          <span aria-hidden>📍</span>
          {item.ipMasked}
        </span>
      )}
    </p>
  );
};

export default ClientMeta;
