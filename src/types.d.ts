import { InvokeArgs } from '@tauri-apps/api/core';

// Core Command type
type Command<
  TInput extends InvokeArgs = InvokeArgs | undefined,
  TOutput = void
> = (input: TInput) => CommandResult<TOutput>;

// Global Command Error type
type CommandError<T = unknown> = {
  code: number;
  message: string;
  data?: T;
};

type CommandResult<T = void> = T & { code: number | 200 };

// Types and interfaces related to Mods
interface ModInfo {
  id: string;
  name: string;
  filePath: string;
  enabled: boolean;
  // Add other relevant fields based on actual Rust struct
}

// Types and interfaces related to Minecraft Versions
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

interface VersionLoader {
  id: string;
  base: 'FABRIC' | 'FORGE';
  // Add other relevant fields if present in Rust struct
  getInstalledId(): string; // Assuming this method exists based on Rust code
}

const VersionBase = {
  FABRIC: 'FABRIC',
  FORGE: 'FORGE',
  NEO_FORGE: 'NEO_FORGE',
  LITE_LOADER: 'LITE_LOADER',
  VANILLA: 'VANILLA',
} as const;

type VersionBaseType = (typeof VersionBase)[keyof typeof VersionBase];

// Types and interfaces related to Configuration
interface LaunchOptions {
  username: string;
  ramUsage: number;
  // Add other launch-related options
}

interface LauncherSettings {
  language: string;
  // Add other settings like theme, etc.
}

interface Config {
  versions: MinecraftVersion[];
  launchOptions: LaunchOptions;
  launcherSettings: LauncherSettings;
  // Add other top-level configuration fields
  write_to_file(): void; // Assuming this method exists based on Rust code
}

// Command definitions using the Command type

// Mod Commands
type ToggleMod = Command<{
  modInfo: ModInfo;
  toggle: boolean;
}>;
type DeleteMod = Command<ModInfo>;
type GetMods = Command<void, ModInfo[]>;

// Version Commands
type LoadCategorizedVersions = Command<
  {
    fabric: boolean;
    forge: boolean;
    neoForge: boolean;
    liteLoader: boolean;
  },
  VersionCategory[]
>;
type GetVersions = Command<void, string[]>;
type GetInstalledVersions = Command<void, string[]>;
type GetNonInstalledVersions = Command<void, string[]>;
type DownloadVersion = Command<{
  appHandle: any; // Replace with actual Tauri AppHandle type if available
  versionLoader: VersionLoader;
}>;
type ReloadVersions = Command<void>;

// Config Commands
type SetUsername = Command<{
  username: string;
}>;
type GetUsername = Command<void, string>;
type SetRamUsage = Command<number>; // Rust uses u64, TypeScript number is fine
type GetRamUsage = Command<void, number>;
type GetTotalRam = Command<void, number>;
type Save = Command<void>;
type SetLanguage = Command<string>;
type GetLanguage = Command<void, string>;

// Game and Profile Commands
type PlayButtonHandler = Command<string>; // selected_version
type GetProfiles = Command<void, string[]>;
type CreateOfflineProfile = Command<{
  username: string;
}>;

// Mod Installation Command
type InstallModFromLocal = Command<void>; // Uses system dialog
