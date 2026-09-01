import type { app } from "@tauri-apps/api";
import {useTranslation} from "react-i18next";
import {useState} from "react";

type AppHandle = typeof app;

interface InvokeError<T = unknown> {
  code: string;
  data?: T;
}
export interface NativeChoice {
  mode: "version_associated" | "custom";
  path: string;
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
  open_mods_folder: {
    args: {
      version: string;
    };
    returns: void;
  };
  debug: {
    args: {
      text: string;
    };
    returns: void;
  };
  get_total_ram: {
    args: undefined;
    returns: number;
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
      ram_usage: number;
    };
    returns: void;
  };
  get_ram_usage: {
    args: undefined;
    returns: number;
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
    args: undefined;
    returns: void;
  };
  download_version: {
    args: {
      versionLoader: VersionLoader;
      name: string;
    };
    returns: void;
  };
  cancel_download: {
    args?: Record<string, never> | undefined;
    returns: void;
  };
// Java native commands
  get_java: {
    args: Record<string, never>;
    returns: NativeChoice;
  };
  set_java: {
    args: {
      java: NativeChoice;
    };
    returns: void;
  };

  // OpenAL native commands
  get_openal: {
    args: Record<string, never>;
    returns: NativeChoice;
  };
  set_openal: {
    args: {
      openal: NativeChoice;
    };
    returns: void;
  };

  // GLFW native commands
  get_glfw: {
    args: Record<string, never>;
    returns: NativeChoice;
  };
  set_glfw: {
    args: {
      glfw: NativeChoice;
    };
    returns: void;
  };
  reload_version_manifest: {
    args: undefined;
    returns: void;
  };
  get_processes: {
    args: undefined;
    returns: string[];
  };
  kill_process: {
    args: {
      selectedProcess: string;
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
      repairMode: boolean;
      profile: string;
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
  should_use_dedicated_gpu: {
    args: undefined;
    returns: boolean;
  };
  set_use_dedicated_gpu: {
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

  // --- PROXY COMMANDS ---
  get_proxy: {
    args: undefined;
    returns: string;
  };
  set_proxy: {
    args: {
      proxy: string;
    };
    returns: void;
  };

  search_for_modrinth_project: {
    args: {
      name: string;
      facets: string;
      index: string;
      offset: number;
      limit: number;
    };
    returns: ModrinthSearchResult;
  };
  get_modrinth_projects: {
    args: {
      projectId: string;
    };
    returns: ModrinthMod;
  };
  list_modrinth_mod_versions: {
    args: {
      projectId: string;
    };
    returns: ModrinthVersion[];
  };
  get_modrinth_mod_dependencies: {
    args: {
      version: ModrinthVersion;
    };
    returns: DependencyTuple[];
  };
  get_modrinth_mod_version_by_id: {
    args: {
      versionId: string;
    };
    returns: ModrinthVersion;
  };
  download_modrinth_mod_version: {
    args: {
      version: ModrinthVersion;
      name: string;
    };
    returns: void;
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
  ramusage: number;
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


export interface ModrinthSearchResultMod {
  project_id: string;
  project_type: string;
  all_project_types: string[];
  title: string;
  description: string;
  author: string;
  categories: string[];
  display_categories: string[];
  versions: string[];
  downloads: number;
  follows: number;
  icon_url: string;
  date_created: string;
  date_modified: string;
  latest_version: string;
  license: string;
  environment: string[];
  disclosure_types: string[];
  gallery: string[];
  slug: string | null;
  author_id: string | null;
  organization: string | null;
  organization_id: string | null;
  featured_gallery: string | null;
  color: number | null;
  client_side: string | null;
  server_side: string | null;
}

export interface ModrinthLicense {
  id: string | null;
  name: string | null;
  url: string | null;
}

export interface ModrinthDonationUrl {
  id: string | null;
  platform: string | null;
  url: string | null;
}

export interface ModrinthGalleryImage {
  url: string;
  featured: boolean;
  title: string | null;
  description: string | null;
  created: string | null;
  ordering: number | null;
}

export interface ModrinthMod {
  id: string;
  team: string | null;
  title: string;
  description: string;
  body: string | null;
  status: string | null;
  project_type: string;
  categories: string[];
  additional_categories: string[];
  environment: string[];
  game_versions: string[];
  loaders: string[];
  versions: string[];
  license: ModrinthLicense | null;
  gallery: ModrinthGalleryImage[];
  donation_urls: ModrinthDonationUrl[];
  published: string | null;
  updated: string | null;
  downloads: number;
  followers: number;
  monetization_status: string | null;
  slug: string | null;
  organization: string | null;
  requested_status: string | null;
  approved: string | null;
  queued: string | null;
  icon_url: string | null;
  raw_icon_url: string | null;
  color: number | null;
  issues_url: string | null;
  source_url: string | null;
  wiki_url: string | null;
  discord_url: string | null;
  client_side: string | null;
  server_side: string | null;
  body_url: string | null;
  moderator_message: string | null;
}

export interface ModrinthSearchResult {
  hits: ModrinthSearchResultMod[];
  offset: number;
  limit: number;
  total_hits: number;
}

export interface ModrinthVersion {
  id: string;
  name: string | null;
  version_number: string | null;
  version_type: string | null;
  date_published: string;
  game_versions?: string[];
  loaders: string[];
  files: any[];
}

export type DependencyTuple = [ModrinthVersion, string];