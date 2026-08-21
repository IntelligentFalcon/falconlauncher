import { Alert01Icon } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { Download, FolderOpen, PackagePlus } from "lucide-react";
import { useTranslation } from "react-i18next";
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
import { useModsActions, useModsState } from "@/context/mods-context";
import { errorText } from "@/messages";
import { ModrinthDownloadModal } from "@/components/blocks/mods/modrinth-download-modal.tsx";

export function ModsHeader() {
  const { t } = useTranslation();

  const {
    installedVersions,
    isImporting,
    isLoadingVersions,
    selectedVersion,
    versionsError,
    isDownloadModalOpen,
  } = useModsState();

  const {
    onImportMod,
    onOpenDownloadModal,
    onOpenFolder,
    setSelectedVersion,
    handleCloseDownloadModal,
  } = useModsActions();

  return (
      <>
        <div className="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-border/40 bg-secondary/30 p-3 shadow-sm backdrop-blur-md">
          <div className="w-full sm:w-64">
            <LoadingSwap isLoading={isLoadingVersions}>
              {versionsError ? (
                  <div className="flex h-10 items-center gap-2 rounded-xl border border-destructive/20 bg-destructive/5 px-3 text-destructive">
                    <HugeiconsIcon
                        className="pointer-events-none shrink-0"
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
                      items={installedVersions}
                      onValueChange={(val) => setSelectedVersion(val ?? "")}
                      value={selectedVersion}
                  >
                    <ComboboxInput
                        className="select-text"
                        placeholder={t("modsHeader.selectVersionPlaceholder")}
                        value={selectedVersion}
                    />
                    <ComboboxContent>
                      <ComboboxEmpty>{t("modsHeader.noVersionsFound")}</ComboboxEmpty>
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

          <div className="flex flex-wrap items-center gap-2">
            <button
                className="flex items-center gap-1.5 rounded-xl border border-border/50 bg-background/60 px-3 py-2 font-medium text-foreground text-xs shadow-sm transition-all hover:bg-secondary"
                onClick={onOpenFolder}
                title={t("modsHeader.openFolderTitle")}
                type="button"
            >
              <FolderOpen className="pointer-events-none h-4 w-4 shrink-0 text-muted-foreground" />
              <span>{t("modsHeader.folderLabel")}</span>
            </button>

            <button
                className={`flex items-center gap-1.5 rounded-xl border border-border/50 px-3 py-2 font-medium text-xs shadow-sm transition-all ${
                    isImporting
                        ? "cursor-not-allowed bg-secondary text-muted-foreground"
                        : "bg-background/60 text-foreground hover:bg-secondary"
                }`}
                disabled={isImporting}
                onClick={onImportMod}
                title={t("modsHeader.importModTitle")}
                type="button"
            >
              <PackagePlus
                  className={`pointer-events-none h-4 w-4 shrink-0 text-muted-foreground ${
                      isImporting ? "animate-pulse" : ""
                  }`}
              />
              <span>{isImporting ? t("modsHeader.importingLabel") : t("modsHeader.importModLabel")}</span>
            </button>

            <ActionButton
                action={async () => onOpenDownloadModal()}
                className="flex items-center gap-1.5 px-3.5 py-2 text-xs"
            >
              <Download className="pointer-events-none h-4 w-4 shrink-0" />
              <span>{t("modsHeader.getModsLabel")}</span>
            </ActionButton>
          </div>
        </div>

        <ModrinthDownloadModal
            isOpen={isDownloadModalOpen}
            onClose={handleCloseDownloadModal}
        />
      </>
  );
}