import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface LocaleStore {
  locale: 'en' | 'fa';
  setLocale: (locale: 'en' | 'fa') => void;
}

export const useLocale = create<LocaleStore>()(
  persist(
    (set) => ({
      locale: 'en',
      setLocale: (locale) => set({ locale }),
    }),
    {
      name: 'locale',
    }
  )
);
