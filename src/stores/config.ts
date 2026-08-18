import { create } from "zustand";
import { persist } from "zustand/middleware";
import {Profile} from "@/invokes";

interface ConfigStore {
  profile: Profile | null;
  setProfile: (profile: Profile | null) => void;
  setVersion: (version: string | null) => void;
  version: string | null;
}

export const useConfig = create<ConfigStore>()(
  persist(
    (set) => ({
      profile: null,
      setProfile: (profile) => set({ profile }),
      setVersion: (version) => set({ version }),
      version: null,
    }),
    {
      name: "config",
    }
  )
);
