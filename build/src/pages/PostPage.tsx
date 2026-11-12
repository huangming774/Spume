import hljs from 'highlight.js';
import 'highlight.js/styles/atom-one-dark.css';
import { useEffect, useMemo, useRef } from 'react';
import { Link, useParams } from 'react-router-dom';
import { Bilingual } from '../components/Bilingual';
import { getArticle } from '../data/content';
import { useI18n } from '../i18n/I18nContext';

function decorateCodeBlocks(root: HTMLElement, copyLabel: string, copiedLabel: string) {
  const pres = root.querySelectorAll<HTMLPreElement>('pre');
  pres.forEach((pre) => {
    if (pre.querySelector(':scope > .code-copy-btn')) return;
    const btn = document.createElement('button');
    btn.type = 'button';
    btn.className = 'code-copy-btn';
    btn.textContent = copyLabel;
    btn.addEventListener('click', () => {
      const code = pre.querySelector('code');
      const text = code ? code.innerText : pre.innerText;
      void navigator.clipboard.writeText(text).then(() => {
        btn.textContent = copiedLabel;
        window.setTimeout(() => {
          btn.textContent = copyLabel;
        }, 2000);
      });
    });
    pre.style.position = 'relative';
    pre.appendChild(btn);
  });
  root.querySelectorAll('pre code').forEach((block) => {
    hljs.highlightElement(block as HTMLElement);
  });
  pres.forEach((pre) => {
    const code = pre.querySelector('code');
    if (code) {
      const cls = code.className || '';
      if (
        /language-plaintext|language-text|language-output|language-none/.test(cls) ||
        !/language-/.test(cls)
      ) {
        pre.classList.add('code-output');
      }
    }
  });
}

export function PostPage() {
  const { slug } = useParams<{ slug: string }>();
  const { lang, t } = useI18n();
  const article = slug ? getArticle(slug) : undefined;
  const proseRef = useRef<HTMLDivElement>(null);

  const bodyHtml = useMemo(
    () => (article ? (lang === 'zh' ? article.bodyHtmlZh : article.bodyHtmlEn) : ''),
    [article, lang],
  );

  const copyLabel = lang === 'zh' ? '复制' : 'Copy';
  const copiedLabel = lang === 'zh' ? '已复制' : 'Copied!';

  useEffect(() => {
    const root = proseRef.current;
    if (!root || !article) return;
    decorateCodeBlocks(root, copyLabel, copiedLabel);
  }, [article, bodyHtml, copyLabel, copiedLabel]);

  if (!article) {
    return (
      <div className="page-main-inner">
        <p className="page-lead">{lang === 'zh' ? '未找到文章。' : 'Post not found.'}</p>
        <Link to="/articles" className="post-back">
          ← {t('navArticle')}
        </Link>
      </div>
    );
  }

  return (
    <article className="page-main-inner post-page">
      <Link to="/articles" className="post-back">
        ← {t('navArticle')}
      </Link>
      <header className="post-header">
        <h1 className="post-title">
          <Bilingual zh={article.title_zh} en={article.title_en} />
        </h1>
        <div className="post-meta article-meta">
          <span>{article.modified ? `${article.date} / 修改于 ${article.modified}` : article.date}</span>
          <span>
            {article.words} {t('metaWords')}
          </span>
          <span>
            {article.reads} {t('metaReads')}
          </span>
          <span>
            {t('metaMinutes')} {article.minutes} {t('metaMinSuffix')}
          </span>
        </div>
      </header>
      <div
        ref={proseRef}
        className={`post-body prose ${lang === 'zh' ? 'i18n-zh' : 'i18n-en'}`}
        dangerouslySetInnerHTML={{ __html: bodyHtml }}
      />
    </article>
  );
}
