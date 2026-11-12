import { useI18n } from '../i18n/I18nContext';

export function Bilingual({ zh, en }: { zh: string; en: string }) {
  const { lang } = useI18n();
  return <>{lang === 'zh' ? zh : en}</>;
}

export function BilingualBlock({ zh, en }: { zh: string; en: string }) {
  const { lang } = useI18n();
  if (lang === 'zh') {
    return (
      <span
        className="i18n-zh"
        dangerouslySetInnerHTML={{ __html: zh.replace(/\n/g, '<br>') }}
      />
    );
  }
  return (
    <span
      className="i18n-en"
      dangerouslySetInnerHTML={{ __html: en.replace(/\n/g, '<br>') }}
    />
  );
}
