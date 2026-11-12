import { moments } from "../data/moments";

const momentDateTime = (moment) => {
  const [month, day] = moment.date.split(".");
  return `${moment.year}-${month.padStart(2, "0")}-${day.padStart(2, "0")}`;
};

const MomentsPage = () => (
  <div className="moments-page">
    <div className="moments-page-bg" aria-hidden="true">
      <span className="moments-page-glow moments-page-glow--rose" />
      <span className="moments-page-glow moments-page-glow--mint" />
      <span className="moments-page-glow moments-page-glow--sun" />
    </div>

    <header className="moments-page-hero">
      <h1>碎语</h1>
      <p className="moments-page-subtitle">过往的点点滴滴，且随风而去吧</p>
    </header>

    <section className="moments-page-list" aria-label="碎语列表">
      {moments.map((moment, index) => (
        <article
          key={`${moment.year}-${moment.date}-${moment.type}`}
          className={`moments-card moments-page-card moments-card--${moment.tone} moments-module--${moment.module}`}
          style={{ "--moment-index": String(index + 1) }}
        >
          <div className="moments-meta">
            <time className="moments-date" dateTime={momentDateTime(moment)}>
              {moment.year} · {moment.date}
            </time>
            <span className="moments-type">{moment.type}</span>
          </div>
          <div className="moments-lines">
            {moment.lines.map((line) => (
              <p key={line}>{line}</p>
            ))}
          </div>
          {moment.image && (
            <figure className="moments-photo">
              <img
                src={moment.image.src}
                alt={moment.image.alt}
                loading="lazy"
              />
            </figure>
          )}
        </article>
      ))}
    </section>
  </div>
);

export default MomentsPage;
