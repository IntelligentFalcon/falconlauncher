import { app } from '@tauri-apps/api';

type AppHandle = typeof app;

type InvokeError<T = unknown> = {
  code: string;
  data?: T;
};

type WithDefaultError<T> = T &
    Record<
        keyof T,
        {
          custom_error: T extends { custom_error: infer E } ? E : {};
        }
    >;

export type Invokes = WithDefaultError<{
  get_fabric_versions: {
    args: {};
    returns: VersionCategory[];
  };
  get_forge_versions: {
    args: {};
    returns: VersionCategory[];
  };
  get_vanilla_versions: {
    args: {};
    returns: VersionCategory[];
  };
  get_versions: {
    args: undefined;
    returns: string[];
  };
  get_mods: {
    args: undefined;
    returns: ModInfo[];
  };
  debug: {
    args: {
      text: string;
    };
    returns: void;
  };
  get_total_ram: {
    args: undefined;
    returns: int;
  };
  set_username: {
    args: {
      username: string;
    };
    returns: void;
  };
  save: {
    args: undefined;
    returns: void;
  };
  set_config: {
    args: {
      config: Config;
    };
    returns: void;
  };
  set_ram_usage: {
    args: {
      ram_usage: int;
    };
    returns: void;
  };
  get_ram_usage: {
    args: undefined;
    returns: int;
  };
  get_username: {
    args: undefined;
    returns: string;
  };
  get_profiles: {
    args: undefined;
    returns: Profile[];
  };
  create_offline_profile: {
    args: {
      username: string;
    };
    returns: void;
  };
  get_installed_versions: {
    args: undefined;
    returns: string[];
  };
  get_non_installed_versions: {
    args: undefined;
    returns: string[];
  };
  set_language: {
    args: {
      lang: string | 'fa' | 'en';
    };
    returns: void;
  };
  get_language: {
    args: undefined;
    returns: string;
  };
  install_mod_from_local: {
    args: {
      app: AppHandle;
    };
    returns: void;
  };
  download_version: {
    args: {
      appHandle: AppHandle;
      versionLoader: VersionLoader;
    };
    returns: void;
  };
  toggle_mod: {
    args: {
      mod_info: ModInfo;
      toggle: boolean;
    };
    returns: void;
  };
  delete_mod: {
    args: { mod_info: ModInfo };
    returns: void;
  };
  play: {
    args: {
      // app: AppHandle;
      selectedVersion: string;
    };
    returns: void;
  };

  // --- NEW COMMANDS ADDED BELOW ---

  get_minimum_ram_usage: {
    args: undefined;
    returns: number;
  };
  get_maximum_ram_usage: {
    args: undefined;
    returns: number;
  };
  set_minimum_ram_usage: {
    args: {
      ramUsage: number; // Verify if your Rust backend expects ramUsage or ram_usage
    };
    returns: void;
  };
  set_maximum_ram_usage: {
    args: {
      ramUsage: number;
    };
    returns: void;
  };
  should_exit_on_launch: {
    args: undefined;
    returns: boolean;
  };
  set_exit_on_launch: {
    args: {
      toggle: boolean;
    };
    returns: void;
  };
  get_available_mirrors: {
    args: undefined;
    returns: Mirror[];
  };
  get_mirror: {
    args: undefined;
    returns: Mirror | null;
  };
  set_mirror: {
    args: {
      mirror: Mirror;
    };
    returns: void;
  };
  import_mirror: {
    args: {
      json: string;
    };
    returns: Mirror[];
  };
}>;

export interface Mirror {
  name: string;
  url: string;
  description: string;
}

export interface MinecraftVersion {
  id: string;
  isInstalled: boolean;
  base: 'FABRIC' | 'FORGE' | 'NEO_FORGE' | 'LITE_LOADER' | 'VANILLA';
  inheritedVersion?: string;
  date: string;
  // Add other relevant fields like type, release time, etc.
}

export enum VersionBase {
  VANILLA,
  FORGE,
  NEOFORGE,
  FABRIC,
  LITELOADER,
}

export interface Profile {
  username: string;
  uuid: string;
  online: boolean;
}

export interface VersionLoader {
  id: string;
  base: VersionBase;
  date: string;
}

export interface VersionCategory {
  name: string; // e.g., "Fabric", "Forge"
  versions: VersionLoader[];
}

export interface Config {
  launchoptions: LaunchOptions;
  launchersettings: LauncherSettings;
  downloadsettings: DownloadSettings;
}

export interface DownloadSettings {
  mirror: string;
}

export interface LauncherSettings {
  language: string;
}

export interface LaunchOptions {
  username: string;
  ramusage: u64;
}

export interface ModInfo {
  path: string;
  modid: string;
  name: string;
  version: string;
  description: string;
  enabled: boolean;
}