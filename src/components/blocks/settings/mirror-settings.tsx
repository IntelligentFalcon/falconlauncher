import {
  FileAddIcon,
  GlobalIcon,
  Tick01Icon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { Button } from "@/components/ui/button";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";

interface Mirror {
  description: string;
  name: string;
  url: string;
}

export function MirrorSettings() {
  const { t } = useTranslation();

  const [localCurrentMirror, setLocalCurrentMirror] = useState<Mirror | null>(
    null
  );
  const [importError, setImportError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const mirrorsQuery = useBackend({ name: "get_available_mirrors" });
  const mirrorQuery = useBackend({ name: "get_mirror" });

  const { mutate: setMirrorMutation } = useBackendMutation({
    name: "set_mirror",
  });
  const { mutateAsync: importMirrorMutation } = useBackendMutation({
    name: "import_mirror",
  });
  const { mutateAsync: saveMutation } = useBackendMutation({ name: "save" });

  const isQueriesLoading = mirrorsQuery.isLoading || mirrorQuery.isLoading;

  const mirrors = mirrorsQuery.data ?? [];
  const currentMirror = localCurrentMirror ?? mirrorQuery.data ?? null;

  const handleSelectMirror = async (mirror: Mirror) => {
    setLocalCurrentMirror(mirror);
    await setMirrorMutation({ mirror });
    await saveMutation(undefined);
  };

  const processJsonString = async (jsonText: string) => {
    try {
      setImportError(null);
      await importMirrorMutation({ json: jsonText });
      await mirrorsQuery.refetch();
      await saveMutation(undefined);
    } catch (err: unknown) {
      setImportError(
        typeof err === "string" ? err : t("mirrorSettings.invalidJsonError")
      );
    }
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const [file] = e.target.files ?? [];
    if (!file) {
      return;
    }

    const reader = new FileReader();
    reader.onload = async (event) => {
      if (event.target?.result && typeof event.target.result === "string") {
        await processJsonString(event.target.result);
      }
    };
    reader.readAsText(file);

    // Reset so the same file can be re-imported if needed
    e.target.value = "";
  };

  return (
    <LoadingSwap className="h-full w-full" isLoading={isQueriesLoading}>
      <div className="max-w-2xl space-y-6">
        <div className="space-y-1">
          <h3 className="flex items-center gap-2 font-semibold text-foreground text-sm">
            <HugeiconsIcon
              className="text-primary"
              icon={GlobalIcon}
              size={16}
            />{" "}
            {t("mirrorSettings.title")}
          </h3>
          <p className="text-muted-foreground text-xs">
            {t("mirrorSettings.description")}
          </p>
        </div>

        <div className="grid grid-cols-1 gap-3">
          {mirrors.map((mirror) => {
            const isSelected = currentMirror?.name === mirror.name;
            return (
              <button
                className={`flex w-full cursor-pointer items-center justify-between gap-4 rounded-xl border p-4 text-left transition-all ${
                  isSelected
                    ? "border-primary bg-primary/10 shadow-primary/5 shadow-sm"
                    : "border-border/40 bg-secondary/20 hover:bg-secondary/40"
                }`}
                key={mirror.name}
                onClick={() => handleSelectMirror(mirror)}
                type="button"
              >
                <div className="min-w-0 flex-1 space-y-1">
                  <div className="truncate font-bold text-xs capitalize">
                    {mirror.name.replace(/_mirror/g, "")}
                  </div>
                  {mirror.description && (
                    <div className="line-clamp-2 text-[11px] text-muted-foreground leading-normal">
                      {mirror.description}
                    </div>
                  )}
                  <div className="truncate font-mono text-[10px] text-muted-foreground/60">
                    {mirror.url}
                  </div>
                </div>
                {isSelected && (
                  <div className="shrink-0 rounded-full bg-primary p-1 text-primary-foreground">
                    <HugeiconsIcon
                      icon={Tick01Icon}
                      size={12}
                      strokeWidth={3}
                    />
                  </div>
                )}
              </button>
            );
          })}
        </div>

        <div className="flex flex-col gap-2">
          <input
            accept=".json"
            className="sr-only"
            onChange={handleFileChange}
            ref={fileInputRef}
            tabIndex={-1}
            type="file"
          />
          <Button
            onClick={() => fileInputRef.current?.click()}
            size="sm"
            type="button"
            variant="outline"
          >
            <HugeiconsIcon data-icon="inline-start" icon={FileAddIcon} />
            {t("mirrorSettings.importButton")}
          </Button>

          {importError && (
            <div className="rounded-md border border-destructive/20 bg-destructive/10 px-3 py-1 font-mono text-[10px] text-destructive">
              ⚠️ {t("mirrorSettings.errorPrefix")}: {importError}
            </div>
          )}
        </div>
      </div>
    </LoadingSwap>
  );
}
