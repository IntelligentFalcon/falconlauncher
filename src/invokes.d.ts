import { app } from '@tauri-apps/api';

type AppHandle = typeof app;

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
  get_categorized_versions: {
    args: {
      forge: boolean;
      fabric: boolean;
      liteLoader: boolean;
      neoForge: boolean;
    };
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
    returns: string[];
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
  get_mods: {
    args: undefined;
    returns: ModInfo[];
  };
  play: {
    args: {
      // app: AppHandle;
      selectedVersion: string;
    };
    returns: void;
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
enum VersionBase {
  VANILLA,
  FORGE,
  NEOFORGE,
  FABRIC,
  LITELOADER,
}
interface VersionLoader {
  id: string;
  base: VersionBase;
  date: string;
}
interface VersionCategory {
  name: string; // e.g., "Fabric", "Forge"
  versions: VersionLoader[];
}

interface Config {
  launchoptions: LaunchOptions;
  launchersettings: LauncherSettings;
  downloadsettings: DownloadSettings;
}
interface DownloadSettings {
  mirror: String;
}
interface LauncherSettings {
  language: String;
}
interface LaunchOptions {
  username: String;
  ramusage: u64;
}
interface ModInfo {
  path: String;
  modid: String;
  name: String;
  version: String;
  description: String;
  enabled: bool;
}
