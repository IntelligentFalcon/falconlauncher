import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import enJSON from './locale/en.json';
import faJSON from './locale/fa.json';
import { useLocale } from '@/stores/locale';

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: enJSON },
    fa: { translation: faJSON },
  }, // Where we're gonna put translations' files
  lng: useLocale.getState().locale, // Set the initial language of the App
});

useLocale.subscribe(({ locale }) => {
  i18n.changeLanguage(locale);
});
