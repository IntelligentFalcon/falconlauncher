import i18n from "i18next";
import {initReactI18next} from "react-i18next";
import {useLocale} from "@/stores/locale";
import enJSON from "./locales/en.json";
import faJSON from "./locales/fa.json";

i18n.use(initReactI18next).init({
    lng: useLocale.getState().locale, // Set the initial language of the App
    resources: {
        en: {translation: enJSON},
        fa: {translation: faJSON},
    }, // Where we're gonna put translations' files
});
useLocale.subscribe(({locale}) => {
    i18n.changeLanguage(locale);
});