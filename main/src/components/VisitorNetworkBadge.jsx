import { useEffect, useState } from "react";

import { measureClientNetwork } from "../services/clientNetworkApi.js";

const REFRESH_MS = 30_000;

const VisitorNetworkBadge = ({ className = "" }) => {
  const [network, setNetwork] = useState({
    addressLabel: "访客",
    regionLabel: "访客",
    latencyMs: null,
    ipMasked: "",
  });

  useEffect(() => {
    let cancelled = false;
    let timer = null;

    const load = async () => {
      try {
        const next = await measureClientNetwork();
        if (!cancelled) setNetwork(next);
      } catch {
        if (!cancelled) {
          setNetwork((current) => ({
            ...current,
            addressLabel: current.addressLabel || current.ipMasked || "访客",
            regionLabel: current.regionLabel || "访客",
            latencyMs: null,
          }));
        }
      }
    };

    load();
    timer = window.setInterval(load, REFRESH_MS);
    return () => {
      cancelled = true;
      if (timer) window.clearInterval(timer);
    };
  }, []);

  const latencyText =
    typeof network.latencyMs === "number" ? `${network.latencyMs}ms` : "--ms";
  const addressText =
    network.addressLabel || network.ipMasked || network.regionLabel || "访客";
  const regionText = network.regionLabel || "访客";
  const title = network.ipMasked
    ? `${regionText} · ${latencyText} · IP ${network.ipMasked}`
    : `${addressText} · ${latencyText}`;

  return (
    <span className={`visitor-network-badge ${className}`} title={title} aria-label={title}>
      <span className="visitor-network-chip visitor-network-chip--address">
        <span className="visitor-network-dot" aria-hidden />
        <span>{addressText}</span>
      </span>
      <span className="visitor-network-chip visitor-network-chip--latency">
        {latencyText}
      </span>
    </span>
  );
};

export default VisitorNetworkBadge;
