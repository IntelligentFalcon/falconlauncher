import { Alert01Icon } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { Power, Trash2 } from "lucide-react";
import { ActionButton } from "@/components/ui/action-button";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import {
    Empty,
    EmptyDescription,
    EmptyMedia,
    EmptyTitle,
} from "@/components/ui/empty";
import { useModsActions, useModsState } from "@/context/mods-context";
import { errorText } from "@/messages";

export function ModsList() {
    const { isLoadingMods, modsError, modsList, selectedVersion } =
        useModsState();
    const { onDeleteMod, onToggleMod } = useModsActions();

    const renderModsContent = () => {
        if (modsError) {
            return (
                <div className="flex flex-1 items-center justify-center">
                    <Empty>
                        <EmptyMedia variant="icon">
                            {/* Added pointer-events-none to prevent icon dragging */}
                            <HugeiconsIcon className="pointer-events-none shrink-0" icon={Alert01Icon} size={24} />
                        </EmptyMedia>
                        <EmptyTitle>{errorText(modsError.code).title}</EmptyTitle>
                        <EmptyDescription>
                            {errorText(modsError.code).description}
                        </EmptyDescription>
                    </Empty>
                </div>
            );
        }

        if (modsList.length === 0) {
            return (
                <div className="flex flex-1 items-center justify-center">
                    <Empty>
                        <EmptyTitle>No mods installed</EmptyTitle>
                        <EmptyDescription>
                            No mods installed for {selectedVersion || "this version"}. Click
                            "Get Mods" or "Import Mod" to add some!
                        </EmptyDescription>
                    </Empty>
                </div>
            );
        }

        return (
            <div className="flex flex-wrap content-start gap-3 pr-1 scrollbar-thin scrollbar-thumb-muted-foreground/20">
                {modsList.map((mod) => (
                    <div
                        className={`flex w-full min-w-[300px] flex-1 items-center justify-between rounded-xl border p-3.5 transition-all duration-200 ${
                            mod.enabled
                                ? "border-border/60 bg-background/80 shadow-sm"
                                : "border-border/30 bg-background/30 opacity-60"
                        }`}
                        key={mod.id || mod.fileName}
                    >
                        <div className="flex min-w-0 items-center gap-3.5 pr-4">
                            <div className="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-lg border border-border/40 bg-secondary">
                                {mod.iconUrl ? (
                                    <img
                                        alt={mod.name}
                                        // Added pointer-events-none and draggable={false} to stop ghost dragging on mod icons
                                        className="pointer-events-none h-full w-full object-cover"
                                        draggable={false}
                                        height={40}
                                        src={mod.iconUrl}
                                        width={40}
                                    />
                                ) : (
                                    <span className="font-bold text-muted-foreground text-xs uppercase">
                    {mod.name?.slice(0, 2) ?? "MD"}
                  </span>
                                )}
                            </div>

                            <div className="min-w-0">
                                <div className="flex items-center gap-2">
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

                        <div className="flex shrink-0 items-center gap-2">
                            <ActionButton
                                action={() => onToggleMod(mod)}
                                className={`h-8 w-8 rounded-lg border-none p-2 transition-all ${
                                    mod.enabled
                                        ? "bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20"
                                        : "bg-muted text-muted-foreground hover:bg-secondary"
                                }`}
                                size="icon"
                                title={mod.enabled ? "Disable Mod" : "Enable Mod"}
                                variant="outline"
                            >
                                {/* Added pointer-events-none shrink-0 */}
                                <Power className="pointer-events-none h-4 w-4 shrink-0" />
                            </ActionButton>

                            <ActionButton
                                action={() => onDeleteMod(mod)}
                                className="h-8 w-8 rounded-lg border-none p-2 text-muted-foreground transition-all hover:bg-destructive/10 hover:text-destructive"
                                size="icon"
                                title="Delete Mod"
                                variant="outline"
                            >
                                {/* Added pointer-events-none shrink-0 */}
                                <Trash2 className="pointer-events-none h-4 w-4 shrink-0" />
                            </ActionButton>
                        </div>
                    </div>
                ))}
            </div>
        );
    };

    return (
        <LoadingSwap className="flex h-full flex-col" isLoading={isLoadingMods}>
            {renderModsContent()}
        </LoadingSwap>
    );
}