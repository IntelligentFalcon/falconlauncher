import { useLocale } from '@/stores/locale';

export function LocaleButton() {
  const { locale, setLocale } = useLocale();

  return (
    <button
      onClick={() => setLocale(locale === 'fa' ? 'en' : 'fa')}
      title="Change Locale"
    >
      {locale === 'fa' ? 'fa' : 'en'}
    </button>
  );
}
