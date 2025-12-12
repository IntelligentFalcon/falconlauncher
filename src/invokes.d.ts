import { InvokeArgs } from '@tauri-apps/api/core';

type InvokeError<T = unknown> = {
  code: number;
  message: string;
  data?: T;
};

type WithDefaultError<T> = T &
  Record<
    keyof T,
    {
      custom_error: T extends { custom_error: infer E } ? E : {};
    }
  >;

type Invokes = WithDefaultError<{
  load_categorized_versions: {
    args: {
      fabric: boolean;
      forge: boolean;
      neoForge: boolean;
      liteLoader: boolean;
    };
    returns: VersionCategory[];
  };
  create_offline_profile: {
    args: {
      username: string;
    };
    returns: void;
  };
  toggle_mod: {
    args: {
      modInfo: ModInfo;
      toggle: boolean;
    };
    returns: void;
  };
  delete_mod: {
    args: ModInfo;
    returns: void;
  };
  get_mods: {
    args: undefined;
    returns: ModInfo[];
  };
}>;

interface MinecraftVersion {
  id: string;
  isInstalled: boolean;
  base: 'FABRIC' | 'FORGE' | 'NEO_FORGE' | 'LITE_LOADER' | 'VANILLA';
  inheritedVersion?: string;
  date: string;
  // Add other relevant fields like type, release time, etc.
}
interface VersionCategory {
  name: string; // e.g., "Fabric", "Forge"
  versions: MinecraftVersion[];
}
