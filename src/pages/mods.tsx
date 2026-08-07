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
import {ModInfo, VersionBase} from '@/invokes';
import { Download, FolderOpen, PackagePlus, Power, Trash2 } from 'lucide-react';
import { useEffect, useState } from 'react';
import { HugeiconsIcon } from '@hugeicons/react';
import { Alert01Icon } from '@hugeicons/core-free-icons';
import {
  Empty,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty';
import { errorText } from '@/messages';

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

  // --- Fetch Versions ---
  const { data: installedVersions, isLoading: isLoadingVersions, error: versionsError } = useBackend({
    name: 'get_versions',
  });

  useEffect(() => {
    if (installedVersions && installedVersions.length > 0 && !selectedVersion) {
      const firstItem= installedVersions[0];
      setSelectedVersion(firstItem);
    }
  }, [installedVersions, selectedVersion]);

  // --- Fetch Mods ---
  const {
    data: fetchedMods,
    isLoading: isLoadingMods,
    refetch: refreshMods,
    error: modsError,
  } = useBackend({
    name: 'get_mods',
    // ✅ Custom queryKey ensures React Query refetches when the selected version changes
    queryKey: ['get_mods', selectedVersion],
    // ✅ If your Rust invoke expects the version as an argument, uncomment the line below:
    // args: { version: selectedVersion },
    enabled: !!selectedVersion, // Don't run the query until we actually have a version selected
  });

  useEffect(() => {
    if (fetchedMods) {
      setModsList(
          fetchedMods.map(
              (mod): ModItem => ({
                id: mod.modId,
                name: mod.name,
                version: mod.version,
                description: mod.description,
                enabled: mod.enabled,
                fileName: mod.path,
              }),
          ),
      );
    } else {
      setModsList([]);
    }
  }, [fetchedMods]);

  // --- Mutations ---
  const { mutateAsync: toggleModBackend } = useBackendMutation({
    name: 'toggle_mod',
  });

  const { mutateAsync: deleteModBackend } = useBackendMutation({
    name: 'delete_mod',
  });

  const { mutateAsync: importModBackend, isPending: isImporting } = useBackendMutation({
    name: 'import_mod_from_local',
  });

  const handleToggleMod = async (mod: ModItem) => {
    const originalModInfo: ModInfo = {
      modId: mod.id,
      name: mod.name,
      version: mod.version,
      description: mod.description,
      enabled: mod.enabled,
      path: mod.fileName,
    };

    setModsList((prev) =>
        prev.map((m) => (m.id === mod.id ? { ...m, enabled: !m.enabled } : m)),
    );

    try {
      await toggleModBackend({
        modInfo: originalModInfo,
        toggle: !mod.enabled,
      });
    } catch (error) {
      setModsList((prev) =>
          prev.map((m) => (m.id === mod.id ? { ...m, enabled: mod.enabled } : m)),
      );
    }
  };

  const handleDeleteMod = async (mod: ModItem) => {
    const originalModInfo: ModInfo = {
      modId: mod.id,
      name: mod.name,
      version: mod.version,
      description: mod.description,
      enabled: mod.enabled,
      path: mod.fileName,
    };

    // Optimistic UI update
    setModsList((prev) => prev.filter((m) => m.id !== mod.id));

    try {
      await deleteModBackend({
        modInfo: originalModInfo,
      });
    } catch (error) {
      // ✅ Revert on failure by refreshing the clean list from the backend
      refreshMods();
    }
  };

  const handleImportMod = async () => {
    try {
      // ✅ `TVarsType` evaluates to `void`, so TypeScript expects exactly 0 arguments.
      // Removed `undefined` to fix strict typing.
      await importModBackend();
      refreshMods();
    } catch (error) {
      // Catch prevents Unhandled Promise crash. Error toast handled globally.
    }
  };

  const handleOpenFolder = async () => {
    console.log('Opening mods directory...');
  };

  const handleOpenDownloadModal = () => {
    console.log('Open download modal...');
  };

  const versionItems =
      (
          installedVersions as
              | Array<{ id?: string; name?: string } | string>
              | undefined
      )?.map((v) => (typeof v === 'string' ? v : v.id || v.name || '')) ?? [];

  return (
      <div className="flex flex-col h-full space-y-4 p-2">
        {/* Top Header Bar */}
        <div className="flex items-center justify-between gap-4 bg-secondary/30 backdrop-blur-md p-3 rounded-2xl border border-border/40 shadow-sm">
          <div className="w-64">
            <LoadingSwap isLoading={isLoadingVersions}>
              {versionsError ? (
                  <div className="flex items-center gap-2 h-10 px-3 border border-destructive/20 rounded-xl bg-destructive/5 text-destructive">
                    <HugeiconsIcon icon={Alert01Icon} size={16} className="shrink-0" />
                    <span className="text-xs font-medium truncate">
                  {errorText(versionsError.code).title}
                </span>
                  </div>
              ) : (
                  <Combobox
                      items={versionItems}
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
              )}
            </LoadingSwap>
          </div>

          <div className="flex items-center space-x-2">
            <button
                onClick={handleOpenFolder}
                title="Open Mods Folder"
                className="flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-xl bg-background/60 hover:bg-secondary border border-border/50 text-foreground transition-all shadow-sm"
            >
              <FolderOpen className="w-4 h-4 text-muted-foreground" />
              <span>Folder</span>
            </button>

            <button
                onClick={handleImportMod}
                disabled={isImporting}
                title="Import .jar file"
                className={`flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-xl border border-border/50 transition-all shadow-sm ${
                    isImporting
                        ? 'bg-secondary text-muted-foreground cursor-not-allowed'
                        : 'bg-background/60 hover:bg-secondary text-foreground'
                }`}
            >
              <PackagePlus className={`w-4 h-4 text-muted-foreground ${isImporting ? 'animate-pulse' : ''}`} />
              <span>{isImporting ? 'Importing...' : 'Import Mod'}</span>
            </button>

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
          <LoadingSwap isLoading={isLoadingMods} className="h-full flex flex-col">
            {modsError ? (
                <div className="flex-1 flex items-center justify-center">
                  <Empty>
                    <EmptyMedia variant="icon">
                      <HugeiconsIcon icon={Alert01Icon} size={24} />
                    </EmptyMedia>
                    <EmptyTitle>
                      {errorText(modsError.code).title}
                    </EmptyTitle>
                    <EmptyDescription>
                      {errorText(modsError.code).description}
                    </EmptyDescription>
                  </Empty>
                </div>
            ) : modsList.length === 0 ? (
                <div className="flex-1 flex items-center justify-center">
                  <Empty>
                    <EmptyTitle>
                      No mods installed
                    </EmptyTitle>
                    <EmptyDescription>
                      No mods installed for {selectedVersion || 'this version'}. Click "Get Mods" or "Import Mod" to add some!
                    </EmptyDescription>
                  </Empty>
                </div>
            ) : (
                <div className="space-y-2.5 overflow-y-auto pr-1 h-full scrollbar-thin scrollbar-thumb-muted-foreground/20">
                  {modsList.map((mod) => (
                      <div
                          key={mod.id || mod.fileName}
                          className={`flex items-center justify-between p-3.5 rounded-xl border transition-all duration-200 ${
                              mod.enabled
                                  ? 'bg-background/80 border-border/60 shadow-sm'
                                  : 'bg-background/30 border-border/30 opacity-60'
                          }`}
                      >
                        <div className="flex items-center space-x-3.5 min-w-0 pr-4">
                          <div className="w-10 h-10 rounded-lg bg-secondary flex items-center justify-center shrink-0 overflow-hidden border border-border/40">
                            {mod.iconUrl ? (
                                <img
                                    src={mod.iconUrl}
                                    alt={mod.name}
                                    className="w-full h-full object-cover"
                                />
                            ) : (
                                <span className="text-xs font-bold uppercase text-muted-foreground">
                          {mod.name?.substring(0, 2) ?? 'MD'}
                        </span>
                            )}
                          </div>

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

                        <div className="flex items-center space-x-2 shrink-0">
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