import { useEffect, useState } from "react";
import { Link } from "react-router-dom";

const HotWeek = () => {
  const [items, setItems] = useState([]);

  useEffect(() => {
    let cancelled = false;
    fetch("/api/client/hot?kind=article", { headers: { Accept: "application/json" } })
      .then((res) => (res.ok ? res.json() : null))
      .then((data) => {
        if (!cancelled && data?.configured && Array.isArray(data.items)) {
          setItems(data.items.slice(0, 6));
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, []);

  if (!items.length) return null;

  return (
    <section className="bg-[#fff8f1] py-14">
      <div className="container mx-auto px-5 md:px-10">
        <p className="font-general text-xs uppercase tracking-[0.45em] text-[#b76e79]">
          This Week
        </p>
        <h2 className="mt-3 text-3xl font-black text-[#241322] md:text-4xl">本周热门</h2>
        <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {items.map((item, index) => (
            <Link
              key={item.id}
              to={item.url}
              className="group rounded-2xl border border-[#f2d8c8] bg-white/70 p-5 transition hover:-translate-y-1 hover:shadow-[0_16px_40px_rgba(183,110,121,0.15)]"
            >
              <span className="font-general text-[10px] uppercase tracking-[0.3em] text-[#b76e79]">
                #{index + 1} · {item.views} 次阅读
              </span>
              <p className="mt-2 line-clamp-2 text-lg font-bold text-[#241322] group-hover:text-[#b76e79]">
                {item.title}
              </p>
            </Link>
          ))}
        </div>
      </div>
    </section>
  );
};

export default HotWeek;
