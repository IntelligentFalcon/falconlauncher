import {
  FileAddIcon,
  GlobalIcon,
  Tick01Icon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import type React from "react";
import { useState } from "react";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";

interface Mirror {
  description: string;
  name: string;
  url: string;
}

export function MirrorSettings() {
  const [localCurrentMirror, setLocalCurrentMirror] = useState<Mirror | null>(
    null
  );
  const [isDragging, setIsDragging] = useState<boolean>(false);
  const [importError, setImportError] = useState<string | null>(null);

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
        typeof err === "string"
          ? err
          : "Invalid JSON format or missing required properties"
      );
      console.error("Mirror injection runtime error:", err);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const [file] = e.dataTransfer.files;
    if (
      file &&
      (file.type === "application/json" || file.name.endsWith(".json"))
    ) {
      const reader = new FileReader();
      reader.onload = async (event) => {
        if (event.target?.result && typeof event.target.result === "string") {
          await processJsonString(event.target.result);
        }
      };
      reader.readAsText(file);
    } else {
      setImportError("Please drop a valid file ending in .json format");
    }
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
            Asset Repository Mirrors
          </h3>
          <p className="text-muted-foreground text-xs">
            Select or drop explicit index maps to bypass primary servers.
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
        {/* biome-ignore lint/a11y/noNoninteractiveElementInteractions: drag-and-drop dropzone has no keyboard-equivalent interactive element */}
        <section
          aria-label="Drag & drop mirror JSON configuration manifest"
          className={`flex flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed p-8 text-center transition-all ${
            isDragging
              ? "border-primary bg-primary/5 text-primary"
              : "border-border/60 bg-secondary/10 text-muted-foreground hover:bg-secondary/20"
          }`}
          onDragLeave={() => setIsDragging(false)}
          onDragOver={(e) => {
            e.preventDefault();
            setIsDragging(true);
          }}
          onDrop={handleDrop}
        >
          <HugeiconsIcon
            className={
              isDragging
                ? "animate-pulse text-primary"
                : "text-muted-foreground/60"
            }
            icon={FileAddIcon}
            size={24}
          />
          <div className="font-medium text-foreground text-xs">
            Drag & Drop Mirror JSON configuration manifest
          </div>
          <div className="text-[10px]">
            Inject files directly into your filesystem architecture
            configurations
          </div>

          {importError && (
            <div className="mt-2 rounded-md border border-destructive/20 bg-destructive/10 px-3 py-1 font-mono text-[10px] text-destructive">
              ⚠️ Error: {importError}
            </div>
          )}
        </section>
      </div>
    </LoadingSwap>
  );
}
