import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface ConfigStore {
  profile: string | null;
  version: string | null;
  setProfile: (profile: string | null) => void;
  setVersion: (version: string | null) => void;
}

export const useConfig = create<ConfigStore>()(
  persist(
    (set) => ({
      profile: null,
      version: null,
      setProfile: (profile) => set({ profile }),
      setVersion: (version) => set({ version }),
    }),
    {
      name: 'config',
    },
  ),
);
