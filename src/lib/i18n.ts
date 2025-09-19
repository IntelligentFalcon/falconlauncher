import { useLocale } from '@/stores/locale';
import { XMLParser } from 'fast-xml-parser';

const parser = new XMLParser({
  ignoreAttributes: false,
  attributeNamePrefix: '@_',
  isArray: (name) => name === 'string',
});

let translations: Record<string, string> = {};
export async function reloadTranslations(locale: 'en' | 'fa') {
  const res = await fetch(`/${locale}.xml`);
  console.log(locale);

  if (!res.ok) {
    throw new Error(`Could not load ${locale}.xml`);
  }
  const xmlText = await res.text();
  const json = parser.parse(xmlText);

  const entries = json.resources.string;
  const map: Record<string, string> = {};

  if (Array.isArray(entries)) {
    entries.forEach((entry) => {
      map[entry['@_id']] = entry['#text'];
    });
  } else {
    map[entries['@_id']] = entries['#text'];
  }

  translations = map;
}

export function t(text: string) {
  return translations[text] || text;
}

useLocale.subscribe(({ locale }) => {
  reloadTranslations(locale);
});

reloadTranslations(useLocale.getState().locale);
