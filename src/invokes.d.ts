import type { app } from "@tauri-apps/api";

type AppHandle = typeof app;

interface InvokeError<T = unknown> {
  code: string;
  data?: T;
}

type WithDefaultError<T> = T &
  Record<
    keyof T,
    {
      custom_error: T extends { custom_error: infer E } ? E : undefined;
    }
  >;

export type Invokes = WithDefaultError<{
  get_fabric_versions: {
    args: undefined;

    returns: VersionCategory[];
  };
  get_forge_versions: {
    args: undefined;
    returns: VersionCategory[];
  };
  get_vanilla_versions: {
    args: undefined;
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
    returns: number; // Note: Changed 'int' to 'number' (TypeScript doesn't have 'int')
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
      ram_usage: number; // Changed 'int' to 'number'
    };
    returns: void;
  };
  get_ram_usage: {
    args: undefined;
    returns: number; // Changed 'int' to 'number'
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
  remove_profile: {
    args: {
      profile: Profile;
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
      lang: string | "fa" | "en";
    };
    returns: void;
  };
  get_language: {
    args: undefined;
    returns: string;
  };
  import_mod_from_local: {
    args: undefined; // AppHandle is injected by Tauri Rust, no frontend arg needed
    returns: void;
  };
  download_version: {
    args: {
      versionLoader: VersionLoader;
      name: string;
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
    args: {
      modInfo: ModInfo;
    };
    returns: void;
  };
  play: {
    args: {
      selectedVersion: string;
      repairMode: boolean,
      profile: string,
    };
    returns: void;
  };
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
      ramUsage: number;
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
  description: string;
  name: string;
  url: string;
}

export interface MinecraftVersion {
  base: "FABRIC" | "FORGE" | "NEO_FORGE" | "LITE_LOADER" | "VANILLA";
  date: string;
  id: string;
  inheritedVersion?: string;
  isInstalled: boolean;
}

export enum VersionBase {
  VANILLA = 0,
  FORGE = 1,
  NEOFORGE = 2,
  FABRIC = 3,
  LITELOADER = 4,
}

export interface Profile {
  online: boolean;
  username: string;
  uuid: string;
}

export interface VersionLoader {
  base: VersionBase;
  date: string;
  id: string;
}

export interface VersionCategory {
  name: string;
  versions: VersionLoader[];
}

export interface Config {
  downloadsettings: DownloadSettings;
  launchersettings: LauncherSettings;
  launchoptions: LaunchOptions;
}

export interface DownloadSettings {
  mirror: string;
}

export interface LauncherSettings {
  language: string;
}

export interface LaunchOptions {
  ramusage: number; // Changed 'u64' to 'number' (TypeScript relies on number or BigInt)
  username: string;
}

export interface ModInfo {
  description: string;
  enabled: boolean;
  modId: string;
  name: string;
  path: string;
  version: string;
}
