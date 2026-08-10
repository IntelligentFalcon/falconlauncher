import { Alert01Icon } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { Download, FolderOpen, PackagePlus, Power, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { ActionButton } from "@/components/ui/action-button";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "@/components/ui/combobox";
import {
  Empty,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";
import type { ModInfo } from "@/invokes";
import { errorText } from "@/messages";

export interface ModItem {
  description: string;
  enabled: boolean;
  fileName: string;
  iconUrl?: string;
  id: string;
  name: string;
  version: string;
}

export default function Mods() {
  const [selectedVersion, setSelectedVersion] = useState<string>("");
  const [modsList, setModsList] = useState<ModItem[]>([]);

  // --- Fetch Versions ---
  const {
    data: installedVersions,
    isLoading: isLoadingVersions,
    error: versionsError,
  } = useBackend({
    name: "get_versions",
  });

  useEffect(() => {
    if (installedVersions && installedVersions.length > 0 && !selectedVersion) {
      const firstItem = installedVersions[0];
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
    // ✅ If your Rust invoke expects the version as an argument, uncomment the line below:
    // args: { version: selectedVersion },
    enabled: !!selectedVersion, // Don't run the query until we actually have a version selected
    name: "get_mods",
    // ✅ Custom queryKey ensures React Query refetches when the selected version changes
    queryKey: ["get_mods", selectedVersion],
  });

  useEffect(() => {
    if (fetchedMods) {
      setModsList(
        fetchedMods.map(
          (mod): ModItem => ({
            description: mod.description,
            enabled: mod.enabled,
            fileName: mod.path,
            id: mod.modId,
            name: mod.name,
            version: mod.version,
          })
        )
      );
    } else {
      setModsList([]);
    }
  }, [fetchedMods]);

  // --- Mutations ---
  const { mutateAsync: toggleModBackend } = useBackendMutation({
    name: "toggle_mod",
  });

  const { mutateAsync: deleteModBackend } = useBackendMutation({
    name: "delete_mod",
  });

  const { mutateAsync: importModBackend, isPending: isImporting } =
    useBackendMutation({
      name: "import_mod_from_local",
    });

  const handleToggleMod = async (mod: ModItem) => {
    const originalModInfo: ModInfo = {
      description: mod.description,
      enabled: mod.enabled,
      modId: mod.id,
      name: mod.name,
      path: mod.fileName,
      version: mod.version,
    };

    setModsList((prev) =>
      prev.map((m) => (m.id === mod.id ? { ...m, enabled: !m.enabled } : m))
    );

    try {
      await toggleModBackend({
        modInfo: originalModInfo,
        toggle: !mod.enabled,
      });
    } catch (error) {
      setModsList((prev) =>
        prev.map((m) => (m.id === mod.id ? { ...m, enabled: mod.enabled } : m))
      );
    }
  };

  const handleDeleteMod = async (mod: ModItem) => {
    const originalModInfo: ModInfo = {
      description: mod.description,
      enabled: mod.enabled,
      modId: mod.id,
      name: mod.name,
      path: mod.fileName,
      version: mod.version,
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
    console.log("Opening mods directory...");
  };

  const handleOpenDownloadModal = () => {
    console.log("Open download modal...");
  };

  const versionItems =
    (
      installedVersions as
        | Array<{ id?: string; name?: string } | string>
        | undefined
    )?.map((v) => (typeof v === "string" ? v : v.id || v.name || "")) ?? [];

  return (
    <div className="flex h-full flex-col space-y-4 p-2">
      {/* Top Header Bar */}
      <div className="flex items-center justify-between gap-4 rounded-2xl border border-border/40 bg-secondary/30 p-3 shadow-sm backdrop-blur-md">
        <div className="w-64">
          <LoadingSwap isLoading={isLoadingVersions}>
            {versionsError ? (
              <div className="flex h-10 items-center gap-2 rounded-xl border border-destructive/20 bg-destructive/5 px-3 text-destructive">
                <HugeiconsIcon
                  className="shrink-0"
                  icon={Alert01Icon}
                  size={16}
                />
                <span className="truncate font-medium text-xs">
                  {errorText(versionsError.code).title}
                </span>
              </div>
            ) : (
              <Combobox
                autoHighlight
                items={versionItems}
                onValueChange={(val) => setSelectedVersion(val ?? "")}
                value={selectedVersion}
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
            className="flex items-center gap-1.5 rounded-xl border border-border/50 bg-background/60 px-3 py-2 font-medium text-foreground text-xs shadow-sm transition-all hover:bg-secondary"
            onClick={handleOpenFolder}
            title="Open Mods Folder"
          >
            <FolderOpen className="h-4 w-4 text-muted-foreground" />
            <span>Folder</span>
          </button>

          <button
            className={`flex items-center gap-1.5 rounded-xl border border-border/50 px-3 py-2 font-medium text-xs shadow-sm transition-all ${
              isImporting
                ? "cursor-not-allowed bg-secondary text-muted-foreground"
                : "bg-background/60 text-foreground hover:bg-secondary"
            }`}
            disabled={isImporting}
            onClick={handleImportMod}
            title="Import .jar file"
          >
            <PackagePlus
              className={`h-4 w-4 text-muted-foreground ${isImporting ? "animate-pulse" : ""}`}
            />
            <span>{isImporting ? "Importing..." : "Import Mod"}</span>
          </button>

          <ActionButton
            action={async () => handleOpenDownloadModal()}
            className="flex items-center gap-1.5 px-3.5 py-2 text-xs"
          >
            <Download className="h-4 w-4" />
            <span>Get Mods</span>
          </ActionButton>
        </div>
      </div>

      {/* Main Mods List Area */}
      <div className="flex flex-1 flex-col overflow-hidden overflow-y-auto rounded-2xl border border-border/40 bg-secondary/20 p-4">
        <LoadingSwap className="flex h-full flex-col" isLoading={isLoadingMods}>
          {modsError ? (
            <div className="flex flex-1 items-center justify-center">
              <Empty>
                <EmptyMedia variant="icon">
                  <HugeiconsIcon icon={Alert01Icon} size={24} />
                </EmptyMedia>
                <EmptyTitle>{errorText(modsError.code).title}</EmptyTitle>
                <EmptyDescription>
                  {errorText(modsError.code).description}
                </EmptyDescription>
              </Empty>
            </div>
          ) : modsList.length === 0 ? (
            <div className="flex flex-1 items-center justify-center">
              <Empty>
                <EmptyTitle>No mods installed</EmptyTitle>
                <EmptyDescription>
                  No mods installed for {selectedVersion || "this version"}.
                  Click "Get Mods" or "Import Mod" to add some!
                </EmptyDescription>
              </Empty>
            </div>
          ) : (
            <div className="scrollbar-thin scrollbar-thumb-muted-foreground/20 space-y-2.5 pr-1">
              {modsList.map((mod) => (
                <div
                  className={`flex items-center justify-between rounded-xl border p-3.5 transition-all duration-200 ${
                    mod.enabled
                      ? "border-border/60 bg-background/80 shadow-sm"
                      : "border-border/30 bg-background/30 opacity-60"
                  }`}
                  key={mod.id || mod.fileName}
                >
                  <div className="flex min-w-0 items-center space-x-3.5 pr-4">
                    <div className="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border/40 bg-secondary">
                      {mod.iconUrl ? (
                        <img
                          alt={mod.name}
                          className="h-full w-full object-cover"
                          src={mod.iconUrl}
                        />
                      ) : (
                        <span className="font-bold text-muted-foreground text-xs uppercase">
                          {mod.name?.substring(0, 2) ?? "MD"}
                        </span>
                      )}
                    </div>

                    <div className="min-w-0">
                      <div className="flex items-center space-x-2">
                        <h4 className="truncate font-semibold text-foreground text-xs">
                          {mod.name}
                        </h4>
                        <span className="rounded bg-secondary px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
                          v{mod.version}
                        </span>
                      </div>
                      <p className="mt-0.5 max-w-md truncate text-[11px] text-muted-foreground">
                        {mod.description || "No description provided."}
                      </p>
                    </div>
                  </div>

                  <div className="flex shrink-0 items-center space-x-2">
                    <button
                      className={`rounded-lg p-2 transition-all ${
                        mod.enabled
                          ? "bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20"
                          : "bg-muted text-muted-foreground hover:bg-secondary"
                      }`}
                      onClick={() => handleToggleMod(mod)}
                      title={mod.enabled ? "Disable Mod" : "Enable Mod"}
                    >
                      <Power className="h-4 w-4" />
                    </button>

                    <button
                      className="rounded-lg p-2 text-muted-foreground transition-all hover:bg-destructive/10 hover:text-destructive"
                      onClick={() => handleDeleteMod(mod)}
                      title="Delete Mod"
                    >
                      <Trash2 className="h-4 w-4" />
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
