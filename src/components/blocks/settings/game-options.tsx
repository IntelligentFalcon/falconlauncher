import {
  CpuIcon,
  GameController01Icon,
  ToggleOffIcon,
  ToggleOnIcon
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { useMemo, useState } from "react";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";

export function GameOptions() {
  // RAM States
  const minLimit = 1024;
  const [localMinRam, setLocalMinRam] = useState<number | null>(null);
  const [localMaxRam, setLocalMaxRam] = useState<number | null>(null);

  // Toggle States
  const [localExitOnLaunch, setLocalExitOnLaunch] = useState<boolean | null>(null);
  const [localUseDedicatedGpu, setLocalUseDedicatedGpu] = useState<boolean | null>(null);

  // Queries
  const minRamQuery = useBackend({ name: "get_minimum_ram_usage" });
  const maxRamQuery = useBackend({ name: "get_maximum_ram_usage" });
  const totalRamQuery = useBackend({ name: "get_total_ram" });
  const exitOnLaunchQuery = useBackend({ name: "should_exit_on_launch" });
  const dedicatedGpuQuery = useBackend({ name: "should_use_dedicated_gpu" });

  // Mutations
  const { mutateAsync: setMinRamMutation } = useBackendMutation({ name: "set_minimum_ram_usage" });
  const { mutateAsync: setMaxRamMutation } = useBackendMutation({ name: "set_maximum_ram_usage" });
  const { mutateAsync: setExitMutation } = useBackendMutation({ name: "set_exit_on_launch" });
  const { mutateAsync: setDedicatedGpuMutation } = useBackendMutation({ name: "set_use_dedicated_gpu" });
  const { mutateAsync: saveMutation } = useBackendMutation({ name: "save" });

  const isQueriesLoading =
      minRamQuery.isLoading ||
      maxRamQuery.isLoading ||
      totalRamQuery.isLoading ||
      exitOnLaunchQuery.isLoading ||
      dedicatedGpuQuery.isLoading;

  const minRam = localMinRam ?? (minRamQuery.data as number) ?? 2048;
  const maxRam = localMaxRam ?? (maxRamQuery.data as number) ?? 4096;
  const maxLimit = (totalRamQuery.data as number) ?? 16_384;

  const exitOnLaunch = localExitOnLaunch ?? (exitOnLaunchQuery.data as boolean) ?? false;
  const useDedicatedGpu = localUseDedicatedGpu ?? (dedicatedGpuQuery.data as boolean) ?? false;

  const sliderTrackStyle = useMemo(() => {
    const totalRange = maxLimit - minLimit;
    const minPercent = ((minRam - minLimit) / totalRange) * 100;
    const maxPercent = ((maxRam - minLimit) / totalRange) * 100;

    return {
      background: `linear-gradient(to right, #27272a 0%, #27272a ${minPercent}%, #0f766e ${minPercent}%, #0f766e ${maxPercent}%, #27272a ${maxPercent}%, #27272a 100%)`,
    };
  }, [minRam, maxRam, maxLimit, minLimit]);

  const handleMinMaxRamChange = async (type: "min" | "max", value: number) => {
    if (type === "min") {
      const targetMin = Math.min(value, maxRam);
      setLocalMinRam(targetMin);
      await setMinRamMutation({ ramUsage: targetMin });
    } else {
      const targetMax = Math.max(value, minRam);
      setLocalMaxRam(targetMax);
      await setMaxRamMutation({ ramUsage: targetMax });
    }
    await saveMutation(undefined);
  };

  const handleExitToggle = async () => {
    const nextState = !exitOnLaunch;
    setLocalExitOnLaunch(nextState);
    await setExitMutation({ toggle: nextState });
    await saveMutation(undefined);
  };

  const handleDedicatedGpuToggle = async () => {
    const nextState = !useDedicatedGpu;
    setLocalUseDedicatedGpu(nextState);
    await setDedicatedGpuMutation({ toggle: nextState });
    await saveMutation(undefined);
  };

  return (
      <LoadingSwap className="h-full w-full" isLoading={isQueriesLoading}>
        <div className="max-w-xl space-y-8">

          {/* Runtime Preferences Section */}
          <div className="space-y-4">
            <div className="space-y-1">
              <h3 className="flex items-center gap-2 font-semibold text-foreground text-sm">
                <HugeiconsIcon className="text-primary" icon={GameController01Icon} size={16} />{" "}
                Runtime Preferences
              </h3>
              <p className="text-muted-foreground text-xs">
                Modify automated window cloaking parameters and game behavior.
              </p>
            </div>

            <div className="flex flex-col gap-3">
              <div className="flex items-center justify-between rounded-xl border border-border/40 bg-secondary/20 p-4">
                <div>
                  <div className="font-semibold text-xs">
                    Exit Launcher on Launch
                  </div>
                  <div className="text-[11px] text-muted-foreground">
                    Kills the application runtime once the subprocess completes
                    assembly boot.
                  </div>
                </div>
                <button
                    className="relative border-none p-0 text-muted-foreground outline-none transition-colors hover:text-foreground"
                    onClick={handleExitToggle}
                    type="button"
                >
                  <HugeiconsIcon
                      className={
                        exitOnLaunch ? "text-primary" : "text-muted-foreground/60"
                      }
                      icon={exitOnLaunch ? ToggleOnIcon : ToggleOffIcon}
                      size={32}
                  />
                </button>
              </div>

              <div className="flex items-center justify-between rounded-xl border border-border/40 bg-secondary/20 p-4">
                <div>
                  <div className="font-semibold text-xs">
                    Use Dedicated GPU
                  </div>
                  <div className="text-[11px] text-muted-foreground">
                    Forces the game to run on the dedicated graphics card if available (e.g., Linux Optimus systems).
                  </div>
                </div>
                <button
                    className="relative border-none p-0 text-muted-foreground outline-none transition-colors hover:text-foreground"
                    onClick={handleDedicatedGpuToggle}
                    type="button"
                >
                  <HugeiconsIcon
                      className={
                        useDedicatedGpu ? "text-primary" : "text-muted-foreground/60"
                      }
                      icon={useDedicatedGpu ? ToggleOnIcon : ToggleOffIcon}
                      size={32}
                  />
                </button>
              </div>
            </div>
          </div>

          {/* Memory Allocation Section */}
          <div className="space-y-4">
            <div className="space-y-1">
              <h3 className="flex items-center gap-2 font-semibold text-foreground text-sm">
                <HugeiconsIcon className="text-primary" icon={CpuIcon} size={16} />{" "}
                Memory Allocation (RAM)
              </h3>
              <p className="text-muted-foreground text-xs">
                Adjust system memory parameters provisioned for game executions.
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
                <div
                    className="absolute top-0 bottom-0 z-10 m-auto h-1.5 w-full rounded-lg transition-[background] duration-75"
                    style={sliderTrackStyle}
                />
              </div>
            </div>
          </div>

        </div>
      </LoadingSwap>
  );
}