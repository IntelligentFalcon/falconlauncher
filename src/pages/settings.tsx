import { ActionButton } from '@/components/ui/action-button';
import { LoadingSwap } from '@/components/ui/animated/swapper';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import { HugeiconsIcon } from '@hugeicons/react';
import {
  Settings01Icon,
  GameController01Icon,
  Download02Icon,
  CpuIcon,
  ToggleOnIcon,
  ToggleOffIcon,
  GlobalIcon,
  FileAddIcon,
  Tick01Icon,
} from '@hugeicons/core-free-icons';
import React, { useEffect, useState, useMemo } from 'react';

// Adjust this import path to point to the file where you exported `useBackend` and `useBackendMutation`
import { useBackend, useBackendMutation } from '@/hooks/use-backend';

interface Mirror {
  name: string;
  url: string;
  description: string;
}

export default function Settings() {
  // We'll manage a local isLoading for the initial sync of all settings
  const [isInitializing, setIsInitializing] = useState<boolean>(true);

  // Unified RAM Settings State
  const minLimit = 1024;
  const maxLimit = 16384;
  const [minRam, setMinRam] = useState<number>(2048);
  const [maxRam, setMaxRam] = useState<number>(4096);

  // Game Options State
  const [language, setLanguage] = useState<string>('en');
  const [exitOnLaunch, setExitOnLaunch] = useState<boolean>(false);

  // Mirror State
  const [mirrors, setMirrors] = useState<Mirror[]>([]);
  const [currentMirror, setCurrentMirror] = useState<Mirror | null>(null);
  const [isDragging, setIsDragging] = useState<boolean>(false);
  const [importError, setImportError] = useState<string | null>(null);

  // --- Queries (using useBackend) ---
  const minRamQuery = useBackend({ name: 'get_minimum_ram_usage' });
  const maxRamQuery = useBackend({ name: 'get_maximum_ram_usage' });
  const langQuery = useBackend({ name: 'get_language' });
  const exitOnLaunchQuery = useBackend({ name: 'should_exit_on_launch' });
  const mirrorsQuery = useBackend({ name: 'get_available_mirrors' });
  const mirrorQuery = useBackend({ name: 'get_mirror' });

  // --- Mutations (using useBackendMutation) ---
  // The global onError inside useBackendMutation will automatically handle Tauri notifications
  const { mutateAsync: setMinRamMutation } = useBackendMutation({ name: 'set_minimum_ram_usage' });
  const { mutateAsync: setMaxRamMutation } = useBackendMutation({ name: 'set_maximum_ram_usage' });
  const { mutateAsync: setLangMutation } = useBackendMutation({ name: 'set_language' });
  const { mutateAsync: setExitMutation } = useBackendMutation({ name: 'set_exit_on_launch' });
  const { mutate: setMirrorMutation } = useBackendMutation({ name: 'set_mirror' });
  const { mutateAsync: importMirrorMutation } = useBackendMutation({ name: 'import_mirror' });

  // Cast variable parameter dynamically depending on whether save needs args in your Invokes
  const { mutateAsync: saveMutation } = useBackendMutation({ name: 'save' });

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
      if (minRamQuery.data !== undefined) setMinRam(minRamQuery.data);
      if (maxRamQuery.data !== undefined) setMaxRam(maxRamQuery.data);
      if (langQuery.data !== undefined) setLanguage(langQuery.data);
      if (exitOnLaunchQuery.data !== undefined) setExitOnLaunch(exitOnLaunchQuery.data);
      if (mirrorsQuery.data !== undefined) setMirrors(mirrorsQuery.data);
      if (mirrorQuery.data !== undefined) setCurrentMirror(mirrorQuery.data);

      setIsInitializing(false);
    }
  }, [
    isQueriesLoading,
    minRamQuery.data,
    maxRamQuery.data,
    langQuery.data,
    exitOnLaunchQuery.data,
    mirrorsQuery.data,
    mirrorQuery.data
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
  const handleMinMaxRamChange = async (type: 'min' | 'max', value: number) => {
    if (type === 'min') {
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
          typeof err === 'string'
              ? err
              : 'Invalid JSON format or missing required properties',
      );
      console.error('Mirror injection runtime error:', err);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const file = e.dataTransfer.files[0];
    if (
        file &&
        (file.type === 'application/json' || file.name.endsWith('.json'))
    ) {
      const reader = new FileReader();
      reader.onload = async (event) => {
        if (event.target?.result) {
          await processJsonString(event.target.result as string);
        }
      };
      reader.readAsText(file);
    } else {
      setImportError('Please drop a valid file ending in .json format');
    }
  };

  return (
      <Tabs defaultValue="launcher">
        {/* Horizontal Navigation Header Layout */}
        <TabsList>
          {[
            { id: 'launcher', label: 'Launcher Settings', icon: Settings01Icon },
            { id: 'game', label: 'Game Options', icon: GameController01Icon },
            { id: 'mirror', label: 'Mirrors', icon: Download02Icon },
          ].map((tab) => (
              <TabsTrigger key={tab.id} value={tab.id}>
                <HugeiconsIcon icon={tab.icon} strokeWidth={2} size={16} />
                <span className="text-xs font-medium">{tab.label}</span>
              </TabsTrigger>
          ))}
        </TabsList>

        {/* Config Panels Panel Box */}
        <div className="flex-1 min-h-0 bg-background/40 border border-border/60 rounded-2xl p-6 overflow-y-auto">
          <LoadingSwap isLoading={isInitializing} className="h-full w-full">
            {/* Panel A: Launcher Options */}
            <TabsContent value="launcher">
              <div className="space-y-6 max-w-xl">
                <div className="space-y-1">
                  <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
                    <HugeiconsIcon
                        icon={CpuIcon}
                        size={16}
                        className="text-primary"
                    />{' '}
                    Memory Allocation (RAM)
                  </h3>
                  <p className="text-xs text-muted-foreground">
                    Adjust system memory parameters provisioned for game
                    executions.
                  </p>
                </div>

                <div className="bg-secondary/30 border border-border/40 p-5 rounded-xl space-y-6">
                  <div className="flex justify-between items-center text-xs font-mono border-b border-border/30 pb-3">
                    <div className="flex flex-col">
                    <span className="text-muted-foreground text-[10px] uppercase font-sans font-bold tracking-wider">
                      Min allocation
                    </span>
                      <span className="text-primary font-bold text-sm">
                      {minRam} MB (~{(minRam / 1024).toFixed(1)} GB)
                    </span>
                    </div>
                    <div className="flex flex-col items-end">
                    <span className="text-muted-foreground text-[10px] uppercase font-sans font-bold tracking-wider">
                      Max allocation
                    </span>
                      <span className="text-emerald-400 font-bold text-sm">
                      {maxRam} MB (~{(maxRam / 1024).toFixed(1)} GB)
                    </span>
                    </div>
                  </div>

                  <div className="relative w-full pt-4 pb-2 h-6 flex items-center">
                    {/* Slider Minimum Handle Track */}
                    <input
                        type="range"
                        min={minLimit}
                        max={maxLimit}
                        step={512}
                        value={minRam}
                        onChange={(e) =>
                            handleMinMaxRamChange('min', Number(e.target.value))
                        }
                        className="absolute pointer-events-none appearance-none w-full bg-transparent top-0 bottom-0 m-auto h-1 z-30 accent-primary [&::-webkit-slider-thumb]:pointer-events-auto"
                    />
                    {/* Slider Maximum Handle Track */}
                    <input
                        type="range"
                        min={minLimit}
                        max={maxLimit}
                        step={512}
                        value={maxRam}
                        onChange={(e) =>
                            handleMinMaxRamChange('max', Number(e.target.value))
                        }
                        className="absolute pointer-events-none appearance-none w-full bg-transparent top-0 bottom-0 m-auto h-1 z-30 accent-emerald-500 [&::-webkit-slider-thumb]:pointer-events-auto"
                    />
                    {/* Dynamic background track injecting the gradient range between knobs */}
                    <div
                        style={sliderTrackStyle}
                        className="w-full h-1.5 rounded-lg z-10 absolute top-0 bottom-0 m-auto transition-[background] duration-75"
                    />
                  </div>
                </div>
              </div>
            </TabsContent>

            {/* Panel B: Game Options */}
            <TabsContent value="game">
              <div className="space-y-6 max-w-xl">
                <div className="space-y-1">
                  <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
                    <HugeiconsIcon
                        icon={GameController01Icon}
                        size={16}
                        className="text-primary"
                    />{' '}
                    Runtime Preferences
                  </h3>
                  <p className="text-xs text-muted-foreground">
                    Modify interface languages and automated window cloaking
                    parameters.
                  </p>
                </div>

                <div className="space-y-4">
                  <div className="flex items-center justify-between bg-secondary/20 p-4 rounded-xl border border-border/40">
                    <div>
                      <div className="text-xs font-semibold">
                        Interface Language
                      </div>
                      <div className="text-[11px] text-muted-foreground">
                        Swaps system core language string values.
                      </div>
                    </div>
                    <select
                        value={language}
                        onChange={(e) => handleLanguageChange(e.target.value)}
                        className="bg-secondary text-xs text-foreground font-medium py-1 px-3 rounded-lg border border-border/80 outline-none cursor-pointer focus:border-primary"
                    >
                      <option value="en">English (US)</option>
                      <option value="fr">Français</option>
                      <option value="de">Deutsch</option>
                      <option value="fa">فارسی</option>
                    </select>
                  </div>

                  <div className="flex items-center justify-between bg-secondary/20 p-4 rounded-xl border border-border/40">
                    <div>
                      <div className="text-xs font-semibold">
                        Exit Launcher on Launch
                      </div>
                      <div className="text-[11px] text-muted-foreground">
                        Kills the application runtime once the subprocess
                        completes assembly boot.
                      </div>
                    </div>
                    <button
                        onClick={handleExitToggle}
                        className="text-muted-foreground hover:text-foreground transition-colors outline-none border-none p-0 relative"
                    >
                      <HugeiconsIcon
                          icon={exitOnLaunch ? ToggleOnIcon : ToggleOffIcon}
                          size={32}
                          className={
                            exitOnLaunch
                                ? 'text-primary'
                                : 'text-muted-foreground/60'
                          }
                      />
                    </button>
                  </div>
                </div>
              </div>
            </TabsContent>

            {/* Panel C: Mirrors */}
            <TabsContent value="mirror">
              <div className="space-y-6 max-w-2xl">
                <div className="space-y-1">
                  <h3 className="text-sm font-semibold text-foreground flex items-center gap-2">
                    <HugeiconsIcon
                        icon={GlobalIcon}
                        size={16}
                        className="text-primary"
                    />{' '}
                    Asset Repository Mirrors
                  </h3>
                  <p className="text-xs text-muted-foreground">
                    Select or drop explicit index maps to bypass primary servers.
                  </p>
                </div>

                <div className="grid grid-cols-1 gap-3">
                  {mirrors.map((mirror, index) => {
                    const isSelected = currentMirror?.name === mirror.name;
                    return (
                        <div
                            key={index}
                            onClick={() => handleSelectMirror(mirror)}
                            className={`p-4 rounded-xl border transition-all cursor-pointer flex items-center justify-between gap-4 ${
                                isSelected
                                    ? 'bg-primary/10 border-primary shadow-sm shadow-primary/5'
                                    : 'bg-secondary/20 border-border/40 hover:bg-secondary/40'
                            }`}
                        >
                          <div className="min-w-0 flex-1 space-y-1">
                            <div className="text-xs font-bold truncate capitalize">
                              {mirror.name.replace(/_mirror/g, '')}
                            </div>

                            {mirror.description && (
                                <div className="text-[11px] text-muted-foreground leading-normal line-clamp-2">
                                  {mirror.description}
                                </div>
                            )}

                            <div className="text-[10px] text-muted-foreground/60 font-mono truncate">
                              {mirror.url}
                            </div>
                          </div>
                          {isSelected && (
                              <div className="bg-primary text-primary-foreground p-1 rounded-full shrink-0">
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
                    onDragOver={(e) => {
                      e.preventDefault();
                      setIsDragging(true);
                    }}
                    onDragLeave={() => setIsDragging(false)}
                    onDrop={handleDrop}
                    className={`border-2 border-dashed rounded-xl p-8 flex flex-col items-center justify-center text-center gap-2 transition-all ${
                        isDragging
                            ? 'border-primary bg-primary/5 text-primary'
                            : 'border-border/60 bg-secondary/10 hover:bg-secondary/20 text-muted-foreground'
                    }`}
                >
                  <HugeiconsIcon
                      icon={FileAddIcon}
                      size={24}
                      className={
                        isDragging
                            ? 'text-primary animate-pulse'
                            : 'text-muted-foreground/60'
                      }
                  />
                  <div className="text-xs font-medium text-foreground">
                    Drag & Drop Mirror JSON configuration manifest
                  </div>
                  <div className="text-[10px]">
                    Inject files directly into your filesystem architecture
                    configurations
                  </div>

                  {importError && (
                      <div className="mt-2 text-[10px] text-destructive bg-destructive/10 border border-destructive/20 py-1 px-3 rounded-md font-mono">
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