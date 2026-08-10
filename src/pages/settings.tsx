import {
  CpuIcon,
  Download02Icon,
  FileAddIcon,
  GameController01Icon,
  GlobalIcon,
  Settings01Icon,
  Tick01Icon,
  ToggleOffIcon,
  ToggleOnIcon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import type React from "react";
import { useEffect, useMemo, useState } from "react";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

// Adjust this import path to point to the file where you exported `useBackend` and `useBackendMutation`
import { useBackend, useBackendMutation } from "@/hooks/use-backend";

interface Mirror {
  description: string;
  name: string;
  url: string;
}

export default function Settings() {
  // We'll manage a local isLoading for the initial sync of all settings
  const [isInitializing, setIsInitializing] = useState<boolean>(true);

  // Unified RAM Settings State
  const minLimit = 1024;
  const maxLimit = 16_384;
  const [minRam, setMinRam] = useState<number>(2048);
  const [maxRam, setMaxRam] = useState<number>(4096);

  // Game Options State
  const [language, setLanguage] = useState<string>("en");
  const [exitOnLaunch, setExitOnLaunch] = useState<boolean>(false);

  // Mirror State
  const [mirrors, setMirrors] = useState<Mirror[]>([]);
  const [currentMirror, setCurrentMirror] = useState<Mirror | null>(null);
  const [isDragging, setIsDragging] = useState<boolean>(false);
  const [importError, setImportError] = useState<string | null>(null);

  // --- Queries (using useBackend) ---
  const minRamQuery = useBackend({ name: "get_minimum_ram_usage" });
  const maxRamQuery = useBackend({ name: "get_maximum_ram_usage" });
  const langQuery = useBackend({ name: "get_language" });
  const exitOnLaunchQuery = useBackend({ name: "should_exit_on_launch" });
  const mirrorsQuery = useBackend({ name: "get_available_mirrors" });
  const mirrorQuery = useBackend({ name: "get_mirror" });

  // --- Mutations (using useBackendMutation) ---
  // The global onError inside useBackendMutation will automatically handle Tauri notifications
  const { mutateAsync: setMinRamMutation } = useBackendMutation({
    name: "set_minimum_ram_usage",
  });
  const { mutateAsync: setMaxRamMutation } = useBackendMutation({
    name: "set_maximum_ram_usage",
  });
  const { mutateAsync: setLangMutation } = useBackendMutation({
    name: "set_language",
  });
  const { mutateAsync: setExitMutation } = useBackendMutation({
    name: "set_exit_on_launch",
  });
  const { mutate: setMirrorMutation } = useBackendMutation({
    name: "set_mirror",
  });
  const { mutateAsync: importMirrorMutation } = useBackendMutation({
    name: "import_mirror",
  });

  // Cast variable parameter dynamically depending on whether save needs args in your Invokes
  const { mutateAsync: saveMutation } = useBackendMutation({ name: "save" });

  // Compute overall loading status of queries
  const isQueriesLoading =
    minRamQuery.isLoading ||
    maxRamQuery.isLoading ||
    langQuery.isLoading ||
    exitOnLaunchQuery.isLoading ||
    mirrorsQuery.isLoading ||
    mirrorQuery.isLoading;

  // Sync React Query data to local state for instantaneous slider/UI responsiveness
  useEffect(() => {
    if (!isQueriesLoading) {
      if (minRamQuery.data !== undefined) {
        setMinRam(minRamQuery.data);
      }
      if (maxRamQuery.data !== undefined) {
        setMaxRam(maxRamQuery.data);
      }
      if (langQuery.data !== undefined) {
        setLanguage(langQuery.data);
      }
      if (exitOnLaunchQuery.data !== undefined) {
        setExitOnLaunch(exitOnLaunchQuery.data);
      }
      if (mirrorsQuery.data !== undefined) {
        setMirrors(mirrorsQuery.data);
      }
      if (mirrorQuery.data !== undefined) {
        setCurrentMirror(mirrorQuery.data);
      }

      setIsInitializing(false);
    }
  }, [
    isQueriesLoading,
    minRamQuery.data,
    maxRamQuery.data,
    langQuery.data,
    exitOnLaunchQuery.data,
    mirrorsQuery.data,
    mirrorQuery.data,
  ]);

  // Calculate the dynamic filled track background style using a linear gradient
  const sliderTrackStyle = useMemo(() => {
    const totalRange = maxLimit - minLimit;
    const minPercent = ((minRam - minLimit) / totalRange) * 100;
    const maxPercent = ((maxRam - minLimit) / totalRange) * 100;

    return {
      background: `linear-gradient(to right, #27272a 0%, #27272a ${minPercent}%, #0f766e ${minPercent}%, #0f766e ${maxPercent}%, #27272a ${maxPercent}%, #27272a 100%)`,
    };
  }, [minRam, maxRam]);

  // --- Unified RAM Slider Handler ---
  const handleMinMaxRamChange = async (type: "min" | "max", value: number) => {
    if (type === "min") {
      const targetMin = Math.min(value, maxRam);
      setMinRam(targetMin); // Optimistic UI update
      await setMinRamMutation({ ramUsage: targetMin });
    } else {
      const targetMax = Math.max(value, minRam);
      setMaxRam(targetMax); // Optimistic UI update
      await setMaxRamMutation({ ramUsage: targetMax });
    }
    await saveMutation(undefined as any);
  };

  // --- Game Options Handlers ---
  const handleLanguageChange = async (lang: string) => {
    setLanguage(lang); // Optimistic UI update
    await setLangMutation({ lang });
    await saveMutation(undefined as any);
  };

  const handleExitToggle = async () => {
    const nextState = !exitOnLaunch;
    setExitOnLaunch(nextState); // Optimistic UI update
    await setExitMutation({ toggle: nextState });
    await saveMutation(undefined as any);
  };

  // --- Mirror Handlers ---
  const handleSelectMirror = async (mirror: Mirror) => {
    setCurrentMirror(mirror); // Optimistic UI update
    await setMirrorMutation({ mirror });
    await saveMutation(undefined as any);
  };

  const processJsonString = async (jsonText: string) => {
    try {
      setImportError(null);

      await importMirrorMutation({ json: jsonText });

      // Re-fetch the available mirrors now that importing is done
      const { data: updatedMirrors } = await mirrorsQuery.refetch();
      if (updatedMirrors) {
        setMirrors(updatedMirrors);
      }

      await saveMutation(undefined as any);
    } catch (err: any) {
      setImportError(
        typeof err === "string"
          ? err
          : "Invalid JSON format or missing required properties"
      );
      console.error("Mirror injection runtime error:", err);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const file = e.dataTransfer.files[0];
    if (
      file &&
      (file.type === "application/json" || file.name.endsWith(".json"))
    ) {
      const reader = new FileReader();
      reader.onload = async (event) => {
        if (event.target?.result) {
          await processJsonString(event.target.result as string);
        }
      };
      reader.readAsText(file);
    } else {
      setImportError("Please drop a valid file ending in .json format");
    }
  };

  return (
    <Tabs defaultValue="launcher">
      {/* Horizontal Navigation Header Layout */}
      <TabsList>
        {[
          { icon: Settings01Icon, id: "launcher", label: "Launcher Settings" },
          { icon: GameController01Icon, id: "game", label: "Game Options" },
          { icon: Download02Icon, id: "mirror", label: "Mirrors" },
        ].map((tab) => (
          <TabsTrigger key={tab.id} value={tab.id}>
            <HugeiconsIcon icon={tab.icon} size={16} strokeWidth={2} />
            <span className="font-medium text-xs">{tab.label}</span>
          </TabsTrigger>
        ))}
      </TabsList>

      {/* Config Panels Panel Box */}
      <div className="min-h-0 flex-1 overflow-y-auto rounded-2xl border border-border/60 bg-background/40 p-6">
        <LoadingSwap className="h-full w-full" isLoading={isInitializing}>
          {/* Panel A: Launcher Options */}
          <TabsContent value="launcher">
            <div className="max-w-xl space-y-6">
              <div className="space-y-1">
                <h3 className="flex items-center gap-2 font-semibold text-foreground text-sm">
                  <HugeiconsIcon
                    className="text-primary"
                    icon={CpuIcon}
                    size={16}
                  />{" "}
                  Memory Allocation (RAM)
                </h3>
                <p className="text-muted-foreground text-xs">
                  Adjust system memory parameters provisioned for game
                  executions.
                </p>
              </div>

              <div className="space-y-6 rounded-xl border border-border/40 bg-secondary/30 p-5">
                <div className="flex items-center justify-between border-border/30 border-b pb-3 font-mono text-xs">
                  <div className="flex flex-col">
                    <span className="font-bold font-sans text-[10px] text-muted-foreground uppercase tracking-wider">
                      Min allocation
                    </span>
                    <span className="font-bold text-primary text-sm">
                      {minRam} MB (~{(minRam / 1024).toFixed(1)} GB)
                    </span>
                  </div>
                  <div className="flex flex-col items-end">
                    <span className="font-bold font-sans text-[10px] text-muted-foreground uppercase tracking-wider">
                      Max allocation
                    </span>
                    <span className="font-bold text-emerald-400 text-sm">
                      {maxRam} MB (~{(maxRam / 1024).toFixed(1)} GB)
                    </span>
                  </div>
                </div>

                <div className="relative flex h-6 w-full items-center pt-4 pb-2">
                  {/* Slider Minimum Handle Track */}
                  <input
                    className="pointer-events-none absolute top-0 bottom-0 z-30 m-auto h-1 w-full appearance-none bg-transparent accent-primary [&::-webkit-slider-thumb]:pointer-events-auto"
                    max={maxLimit}
                    min={minLimit}
                    onChange={(e) =>
                      handleMinMaxRamChange("min", Number(e.target.value))
                    }
                    step={512}
                    type="range"
                    value={minRam}
                  />
                  {/* Slider Maximum Handle Track */}
                  <input
                    className="pointer-events-none absolute top-0 bottom-0 z-30 m-auto h-1 w-full appearance-none bg-transparent accent-emerald-500 [&::-webkit-slider-thumb]:pointer-events-auto"
                    max={maxLimit}
                    min={minLimit}
                    onChange={(e) =>
                      handleMinMaxRamChange("max", Number(e.target.value))
                    }
                    step={512}
                    type="range"
                    value={maxRam}
                  />
                  {/* Dynamic background track injecting the gradient range between knobs */}
                  <div
                    className="absolute top-0 bottom-0 z-10 m-auto h-1.5 w-full rounded-lg transition-[background] duration-75"
                    style={sliderTrackStyle}
                  />
                </div>
              </div>
            </div>
          </TabsContent>

          {/* Panel B: Game Options */}
          <TabsContent value="game">
            <div className="max-w-xl space-y-6">
              <div className="space-y-1">
                <h3 className="flex items-center gap-2 font-semibold text-foreground text-sm">
                  <HugeiconsIcon
                    className="text-primary"
                    icon={GameController01Icon}
                    size={16}
                  />{" "}
                  Runtime Preferences
                </h3>
                <p className="text-muted-foreground text-xs">
                  Modify interface languages and automated window cloaking
                  parameters.
                </p>
              </div>

              <div className="space-y-4">
                <div className="flex items-center justify-between rounded-xl border border-border/40 bg-secondary/20 p-4">
                  <div>
                    <div className="font-semibold text-xs">
                      Interface Language
                    </div>
                    <div className="text-[11px] text-muted-foreground">
                      Swaps system core language string values.
                    </div>
                  </div>
                  <select
                    className="cursor-pointer rounded-lg border border-border/80 bg-secondary px-3 py-1 font-medium text-foreground text-xs outline-none focus:border-primary"
                    onChange={(e) => handleLanguageChange(e.target.value)}
                    value={language}
                  >
                    <option value="en">English (US)</option>
                    <option value="fr">Français</option>
                    <option value="de">Deutsch</option>
                    <option value="fa">فارسی</option>
                  </select>
                </div>

                <div className="flex items-center justify-between rounded-xl border border-border/40 bg-secondary/20 p-4">
                  <div>
                    <div className="font-semibold text-xs">
                      Exit Launcher on Launch
                    </div>
                    <div className="text-[11px] text-muted-foreground">
                      Kills the application runtime once the subprocess
                      completes assembly boot.
                    </div>
                  </div>
                  <button
                    className="relative border-none p-0 text-muted-foreground outline-none transition-colors hover:text-foreground"
                    onClick={handleExitToggle}
                  >
                    <HugeiconsIcon
                      className={
                        exitOnLaunch
                          ? "text-primary"
                          : "text-muted-foreground/60"
                      }
                      icon={exitOnLaunch ? ToggleOnIcon : ToggleOffIcon}
                      size={32}
                    />
                  </button>
                </div>
              </div>
            </div>
          </TabsContent>

          {/* Panel C: Mirrors */}
          <TabsContent value="mirror">
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
                {mirrors.map((mirror, index) => {
                  const isSelected = currentMirror?.name === mirror.name;
                  return (
                    <div
                      className={`flex cursor-pointer items-center justify-between gap-4 rounded-xl border p-4 transition-all ${
                        isSelected
                          ? "border-primary bg-primary/10 shadow-primary/5 shadow-sm"
                          : "border-border/40 bg-secondary/20 hover:bg-secondary/40"
                      }`}
                      key={index}
                      onClick={() => handleSelectMirror(mirror)}
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
                    </div>
                  );
                })}
              </div>

              <div
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
              </div>
            </div>
          </TabsContent>
        </LoadingSwap>
      </div>
    </Tabs>
  );
}
