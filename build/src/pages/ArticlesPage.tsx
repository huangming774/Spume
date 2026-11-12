import { Link } from 'react-router-dom';
import { Bilingual } from '../components/Bilingual';
import { siteContent } from '../data/content';
import { useI18n } from '../i18n/I18nContext';

export function ArticlesPage() {
  const { t } = useI18n();

  return (
    <div className="page-main-inner">
      <h1 className="page-title">{t('articlesTitle')}</h1>
      <div className="article-list">
        {[...siteContent.articles]
          .sort((a, b) => (a.date < b.date ? 1 : -1))
          .map((a) => (
            <Link key={a.slug} to={`/post/${a.slug}`} className="article-card">
              <h2>
                <Bilingual zh={a.title_zh} en={a.title_en} />
              </h2>
              <p className="excerpt">
                <Bilingual zh={a.excerpt_zh} en={a.excerpt_en} />
              </p>
              <div className="article-meta">
                <span>{a.date}</span>
                <span>
                  {a.words} {t('metaWords')}
                </span>
                <span>
                  {a.reads} {t('metaReads')}
                </span>
                <span>
                  {t('metaMinutes')} {a.minutes} {t('metaMinSuffix')}
                </span>
              </div>
            </Link>
          ))}
      </div>
    </div>
  );
}
