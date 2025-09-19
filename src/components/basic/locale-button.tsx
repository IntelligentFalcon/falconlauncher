import { useLocale } from '@/stores/locale';

export function LocaleButton() {
  const { locale, setLocale } = useLocale();

  return (
    <button
      className="p-1 rounded-full hover:bg-gray-700 transition-colors"
      onClick={() => setLocale(locale === 'fa' ? 'en' : 'fa')}
      title="Change Locale"
    >
      {locale === 'fa' ? 'fa' : 'en'}
    </button>
  );
}
