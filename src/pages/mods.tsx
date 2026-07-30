import { ActionButton } from '@/components/ui/action-button';
import { LoadingSwap } from '@/components/ui/animated/swapper';
import {
    Combobox,
    ComboboxContent,
    ComboboxEmpty,
    ComboboxInput,
    ComboboxItem,
    ComboboxList,
} from '@/components/ui/combobox';
import { useBackend, useBackendMutation } from '@/hooks/use-backend';
import {
    Download,
    FolderOpen,
    PackagePlus,
    Power,
    Trash2,
} from 'lucide-react';
import { useEffect, useState } from 'react';

// Type definitions for Mod structures
export interface ModItem {
    id: string;
    name: string;
    version: string;
    description: string;
    iconUrl?: string;
    enabled: boolean;
    fileName: string;
}

export default function Mods() {
    const [selectedVersion, setSelectedVersion] = useState<string>('');
    const [modsList, setModsList] = useState<ModItem[]>([]);

    // 1. Fetch available installed game versions for the dropdown
    // 🦀 RUST HOOK: Backend should query installed instances/versions in the .minecraft directory
    const { data: installedVersions, isLoading: isLoadingVersions } = useBackend<
        string[]
    >({
        name: 'get_installed_versions',
    });

    // Default to first available version once fetched
    useEffect(() => {
        if (installedVersions && installedVersions.length > 0 && !selectedVersion) {
            setSelectedVersion(installedVersions[0]);
        }
    }, [installedVersions, selectedVersion]);

    // 2. Fetch mods specifically for the active selected version
    // 🦀 RUST HOOK: Backend reads the `/mods` folder for this specific version,
    // reads mod metadata (fabric.mod.json / mods.toml), and returns active vs disabled (.disabled) mods
    const { data: fetchedMods, isLoading: isLoadingMods, refetch: refreshMods } =
        useBackend<ModItem[]>({
            name: 'get_installed_mods',
            args: { version: selectedVersion },
        });

    useEffect(() => {
        if (fetchedMods) {
            setModsList(fetchedMods);
        }
    }, [fetchedMods]);

    // Mutations for Mod Actions
    const { mutateAsync: toggleModBackend } = useBackendMutation({
        name: 'toggle_mod',
    });
    const { mutateAsync: deleteModBackend } = useBackendMutation({
        name: 'delete_mod',
    });
    const { mutateAsync: openFolderBackend } = useBackendMutation({
        name: 'open_mods_folder',
    });
    const { mutateAsync: importModBackend } = useBackendMutation({
        name: 'import_mod_file',
    });

    // Handler: Toggle Enable/Disable
    const handleToggleMod = async (mod: ModItem) => {
        // Optimistic UI update
        setModsList((prev) =>
            prev.map((m) => (m.id === mod.id ? { ...m, enabled: !m.enabled } : m))
        );

        // 🦀 RUST HOOK: Command renames the file (e.g. `mod.jar` <-> `mod.jar.disabled`)
        await toggleModBackend({
            version: selectedVersion,
            fileName: mod.fileName,
            enable: !mod.enabled,
        });
    };

    // Handler: Delete Mod
    const handleDeleteMod = async (mod: ModItem) => {
        setModsList((prev) => prev.filter((m) => m.id !== mod.id));

        // 🦀 RUST HOOK: Command deletes the file from disk permanently
        await deleteModBackend({
            version: selectedVersion,
            fileName: mod.fileName,
        });
    };

    // Handler: Open Mods Directory in OS File Explorer
    const handleOpenFolder = async () => {
        // 🦀 RUST HOOK: Uses `std::process::Command` or `opener` crate to open OS file manager at `/mods` path
        await openFolderBackend({ version: selectedVersion });
    };

    // Handler: Import local .jar file
    const handleImportMod = async () => {
        // 🦀 RUST HOOK: Uses `tauri-plugin-dialog` to pick a .jar file and copy it into the active version's `/mods` directory
        await importModBackend({ version: selectedVersion });
        refreshMods();
    };

    // Handler: Open Download Manager / Browser
    const handleOpenDownloadModal = () => {
        // 🦀 RUST HOOK / FRONTEND: Triggers Modrinth/CurseForge search interface or IPC invoke
        console.log('Open mod downloading flow...');
    };

    return (
        <div className="flex flex-col h-full space-y-4 p-2">
            {/* Top Header Bar */}
            <div className="flex items-center justify-between gap-4 bg-secondary/30 backdrop-blur-md p-3 rounded-2xl border border-border/40 shadow-sm">
        {/* Left: Version Selector Dropdown */}
        <div className="w-64">
    <LoadingSwap isLoading={isLoadingVersions}>
    <Combobox
        items={installedVersions ?? []}
    autoHighlight
    value={selectedVersion}
    onValueChange={(val) => setSelectedVersion(val ?? '')}
>
    <ComboboxInput
        placeholder="Select Game Version"
    value={selectedVersion}
    />
    <ComboboxContent>
    <ComboboxEmpty>No installed versions found.</ComboboxEmpty>
    <ComboboxList>
    {(ver) => (
        <ComboboxItem key={ver} value={ver}>
        {ver}
        </ComboboxItem>
)}
    </ComboboxList>
    </ComboboxContent>
    </Combobox>
    </LoadingSwap>
    </div>

    {/* Right: Action Buttons (Folder, Import, Download) */}
    <div className="flex items-center space-x-2">
        {/* Open Folder Button */}
        <button
    onClick={handleOpenFolder}
    title="Open Mods Folder"
    className="flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-xl bg-background/60 hover:bg-secondary border border-border/50 text-foreground transition-all shadow-sm"
    >
    <FolderOpen className="w-4 h-4 text-muted-foreground" />
        <span>Folder</span>
        </button>

    {/* Import Local Mod Button */}
    <button
        onClick={handleImportMod}
    title="Import .jar file"
    className="flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-xl bg-background/60 hover:bg-secondary border border-border/50 text-foreground transition-all shadow-sm"
    >
    <PackagePlus className="w-4 h-4 text-muted-foreground" />
        <span>Import Mod</span>
    </button>

    {/* Download Mods Button */}
    <ActionButton
        action={async () => handleOpenDownloadModal()}
    className="flex items-center gap-1.5 text-xs px-3.5 py-2"
    >
    <Download className="w-4 h-4" />
        <span>Get Mods</span>
    </ActionButton>
    </div>
    </div>

    {/* Main Mods List Area */}
    <div className="flex-1 bg-secondary/20 rounded-2xl border border-border/40 p-4 overflow-hidden flex flex-col">
    <LoadingSwap isLoading={isLoadingMods} className="h-full">
        {modsList.length === 0 ? (
                <div className="h-full flex flex-col items-center justify-center text-center text-muted-foreground space-y-2">
                <p className="text-sm font-medium">No mods installed for {selectedVersion || 'this version'}.</p>
        <p className="text-xs text-muted-foreground/70">
        Click "Get Mods" or "Import Mod" to add some!
    </p>
    </div>
) : (
        <div className="space-y-2.5 overflow-y-auto pr-1 h-full scrollbar-thin scrollbar-thumb-muted-foreground/20">
            {modsList.map((mod) => (
                    <div
                        key={mod.id}
                className={`flex items-center justify-between p-3.5 rounded-xl border transition-all duration-200 ${
                    mod.enabled
                        ? 'bg-background/80 border-border/60 shadow-sm'
                        : 'bg-background/30 border-border/30 opacity-60'
                }`}
            >
            {/* Left Section: Icon & Info */}
            <div className="flex items-center space-x-3.5 min-w-0 pr-4">
        {/* Mod Icon */}
        <div className="w-10 h-10 rounded-lg bg-secondary flex items-center justify-center shrink-0 overflow-hidden border border-border/40">
        {mod.iconUrl ? (
                    <img
                        src={mod.iconUrl}
                alt={mod.name}
            className="w-full h-full object-cover"
                />
) : (
        <span className="text-xs font-bold uppercase text-muted-foreground">
            {mod.name.substring(0, 2)}
            </span>
    )}
    </div>

    {/* Mod Title, Version & Description */}
    <div className="min-w-0">
    <div className="flex items-center space-x-2">
    <h4 className="text-xs font-semibold text-foreground truncate">
        {mod.name}
        </h4>
        <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-secondary text-muted-foreground">
        v{mod.version}
    </span>
    </div>
    <p className="text-[11px] text-muted-foreground truncate max-w-md mt-0.5">
        {mod.description || 'No description provided.'}
        </p>
        </div>
        </div>

    {/* Right Section: Action Controls */}
    <div className="flex items-center space-x-2 shrink-0">
        {/* Toggle Active Status */}
        <button
    onClick={() => handleToggleMod(mod)}
    title={mod.enabled ? 'Disable Mod' : 'Enable Mod'}
    className={`p-2 rounded-lg transition-all ${
        mod.enabled
            ? 'bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20'
            : 'bg-muted text-muted-foreground hover:bg-secondary'
    }`}
>
    <Power className="w-4 h-4" />
        </button>

    {/* Delete Mod */}
    <button
        onClick={() => handleDeleteMod(mod)}
    title="Delete Mod"
    className="p-2 rounded-lg text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-all"
    >
    <Trash2 className="w-4 h-4" />
        </button>
        </div>
        </div>
))}
    </div>
)}
    </LoadingSwap>
    </div>
    </div>
);
}