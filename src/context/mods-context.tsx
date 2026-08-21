import {
    createContext,
    useCallback,
    useContext,
    useMemo,
    useState,
} from "react";
import type { ModItem } from "@/components/blocks/mods/mod-item";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";
import type { InvokeError, Invokes, ModInfo } from "@/invokes";

interface ModsState {
    installedVersions: string[];
    isImporting: boolean;
    isLoadingMods: boolean;
    isLoadingVersions: boolean;
    modsError: InvokeError<Invokes["get_mods"]["custom_error"]> | null;
    modsList: ModItem[];
    selectedVersion: string;
    versionsError: InvokeError<Invokes["get_versions"]["custom_error"]> | null;
    isDownloadModalOpen: boolean; // <-- Kept as State
}

interface ModsActions {
    onDeleteMod: (mod: ModItem) => Promise<void>;
    onImportMod: () => Promise<void>;
    onOpenDownloadModal: () => void;
    handleCloseDownloadModal: () => void; // <-- Added as Action exactly as you named it
    onOpenFolder: () => void;
    onToggleMod: (mod: ModItem) => Promise<void>;
    setSelectedVersion: (val: string) => void;
}

export const ModsStateContext = createContext<ModsState | null>(null);
export const ModsActionsContext = createContext<ModsActions | null>(null);

export function useModsState() {
    const context = useContext(ModsStateContext);
    if (!context) {
        throw new Error("useModsState must be used within ModsProvider");
    }
    return context;
}

export function useModsActions() {
    const context = useContext(ModsActionsContext);
    if (!context) {
        throw new Error("useModsActions must be used within ModsProvider");
    }
    return context;
}

export function ModsProvider({ children }: { children: React.ReactNode }) {
    const [localSelectedVersion, setLocalSelectedVersion] = useState<
        string | null
    >(null);

    const [isDownloadModalOpen, setIsDownloadModalOpen] = useState(false);

    const {
        data: installedVersions,
        isLoading: isLoadingVersions,
        error: versionsError,
    } = useBackend({ name: "get_versions" });

    const selectedVersion = localSelectedVersion ?? installedVersions?.[0] ?? "";

    const {
        data: fetchedMods,
        isLoading: isLoadingMods,
        refetch: refreshMods,
        error: modsError,
    } = useBackend({
        enabled: !!selectedVersion,
        name: "get_mods",
        queryKey: ["get_mods", selectedVersion],
    });

    const modsList = useMemo(
        () =>
            fetchedMods
                ? fetchedMods.map(
                    (mod): ModItem => ({
                        description: mod.description,
                        enabled: mod.enabled,
                        fileName: mod.path,
                        id: mod.modId,
                        name: mod.name,
                        version: mod.version,
                    })
                )
                : [],
        [fetchedMods]
    );

    const { mutateAsync: toggleModBackend } = useBackendMutation({
        name: "toggle_mod",
    });
    const { mutateAsync: deleteModBackend } = useBackendMutation({
        name: "delete_mod",
    });
    const { mutateAsync: openModsFolderBackend } = useBackendMutation({
        name: "open_mods_folder",
    });

    const { mutateAsync: importModBackend, isPending: isImporting } =
        useBackendMutation({
            name: "import_mod_from_local",
        });

    const handleToggleMod = useCallback(
        async (mod: ModItem) => {
            const originalModInfo: ModInfo = {
                description: mod.description,
                enabled: mod.enabled,
                modId: mod.id,
                name: mod.name,
                path: mod.fileName,
                version: mod.version,
            };

            try {
                await toggleModBackend({
                    modInfo: originalModInfo,
                    toggle: !mod.enabled,
                });
                await refreshMods();
            } catch (e) {
                console.error(e);
            }
        },
        [toggleModBackend, refreshMods]
    );

    const handleDeleteMod = useCallback(
        async (mod: ModItem) => {
            const originalModInfo: ModInfo = {
                description: mod.description,
                enabled: mod.enabled,
                modId: mod.id,
                name: mod.name,
                path: mod.fileName,
                version: mod.version,
            };

            try {
                await deleteModBackend({ modInfo: originalModInfo });
                await refreshMods();
            } catch (e) {
                console.error(e);
            }
        },
        [deleteModBackend, refreshMods]
    );

    const handleImportMod = useCallback(async () => {
        try {
            await importModBackend();
            refreshMods();
        } catch {
            // handled globally
        }
    }, [importModBackend, refreshMods]);

    const handleOpenFolder = useCallback(async () => {
        try {
            await openModsFolderBackend({ version: selectedVersion });
        } catch (e) {
            console.error("Failed to open mods folder:", e);
        }
    }, [openModsFolderBackend, selectedVersion]);

    const handleOpenDownloadModal = useCallback(
        () => setIsDownloadModalOpen(true),
        []
    );

    const handleCloseDownloadModal = useCallback(
        () => setIsDownloadModalOpen(false),
        []
    );

    const actions = useMemo(
        () => ({
            onDeleteMod: handleDeleteMod,
            onImportMod: handleImportMod,
            onOpenDownloadModal: handleOpenDownloadModal,
            handleCloseDownloadModal, // <-- Correctly exported as an action here
            onOpenFolder: handleOpenFolder,
            onToggleMod: handleToggleMod,
            setSelectedVersion: setLocalSelectedVersion,
        }),
        [
            handleDeleteMod,
            handleImportMod,
            handleOpenDownloadModal,
            handleCloseDownloadModal, // <-- Added to dependency array
            handleOpenFolder,
            handleToggleMod,
        ]
    );

    const state = useMemo(
        () => ({
            installedVersions: installedVersions ?? [],
            isImporting,
            isLoadingMods,
            isLoadingVersions,
            modsError,
            modsList,
            selectedVersion,
            versionsError,
            isDownloadModalOpen,
        }),
        [
            installedVersions,
            isImporting,
            isLoadingMods,
            isLoadingVersions,
            modsError,
            modsList,
            selectedVersion,
            versionsError,
            isDownloadModalOpen,
        ]
    );

    return (
        <ModsActionsContext.Provider value={actions}>
            <ModsStateContext.Provider value={state}>
                {children}
            </ModsStateContext.Provider>
        </ModsActionsContext.Provider>
    );
}