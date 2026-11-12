import { useMemo } from 'react';
import { Link } from 'react-router-dom';
import { Bilingual } from '../components/Bilingual';
import { siteContent } from '../data/content';
import { useI18n } from '../i18n/I18nContext';
import { useCardRandom } from '../hooks/useCardRandom';

type TimelineEntry =
  | { kind: 'year'; year: string }
  | {
      kind: 'post';
      slug: string;
      date: string;
      md: string;
      titleZh: string;
      titleEn: string;
    };

function formatMd(isoDate: string) {
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(isoDate);
  if (!m) return isoDate;
  return `${m[2]}-${m[3]}`;
}

export function ArchivesPage() {
  const { t } = useI18n();

  const entries = useMemo(() => {
    const sorted = [...siteContent.articles].sort((a, b) => (a.date < b.date ? 1 : -1));
    const byYear = new Map<string, typeof sorted>();
    sorted.forEach((a) => {
      const y = a.date.slice(0, 4);
      if (!byYear.has(y)) byYear.set(y, []);
      byYear.get(y)!.push(a);
    });
    const years = [...byYear.keys()].sort((a, b) => b.localeCompare(a));
    const out: TimelineEntry[] = [];
    years.forEach((year) => {
      out.push({ kind: 'year', year });
      byYear.get(year)!.forEach((a) => {
        out.push({
          kind: 'post',
          slug: a.slug,
          date: a.date,
          md: formatMd(a.date),
          titleZh: a.title_zh,
          titleEn: a.title_en,
        });
      });
    });
    return out;
  }, []);

  useCardRandom([entries.length]);

  return (
    <div className="page-main-inner page-art page-art--neat">
      <h1 className="page-title page-title--art">{t('archiveTitle')}</h1>
      <p className="page-lead">{t('archiveLead')}</p>
      <div className="archive-timeline">
        {entries.map((e, i) =>
          e.kind === 'year' ? (
            <div key={`y-${e.year}`} className="timeline-year">
              <span>{e.year}</span>
            </div>
          ) : (
            <div key={`${e.slug}-${i}`} className="timeline-item">
              <Link to={`/post/${e.slug}`} className="timeline-card moment">
                <div className="row">
                  <span className="timeline-date">{e.md}</span>
                  <span className="timeline-title">
                    <Bilingual zh={e.titleZh} en={e.titleEn} />
                  </span>
                  <span className="timeline-tag">{t('tagDone')}</span>
                </div>
              </Link>
            </div>
          ),
        )}
      </div>
    </div>
  );
}
