import {
  Alert01Icon,
  ArrowLeft02Icon,
  CheckmarkBadge01Icon,
  Download01Icon,
  PackageIcon,
  Settings01Icon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { listen } from "@tauri-apps/api/event";
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
import type { VersionCategory, VersionLoader } from "@/invokes";
import { errorText } from "@/messages";

type LoaderType = "vanilla" | "fabric" | "forge";
type WizardStep = 1 | 2 | 3;

export default function InstallerWizard() {
  // --- Wizard State ---
  const [step, setStep] = useState<WizardStep>(1);
  const [activeLoader, setActiveLoader] = useState<LoaderType>("vanilla");

  // --- Configuration State ---
  const [activeMajorVersion, setActiveMajorVersion] = useState<string>("");
  const [activeVersion, setActiveVersion] = useState<VersionLoader | null>(
    null
  );
  const [instanceName, setInstanceName] = useState<string>("");

  // --- Installation State ---
  const [progress, setProgress] = useState<number | null>(null);
  const [statusText, setStatusText] = useState<string>("");
  const [isDone, setIsDone] = useState<boolean>(false);

  // --- Dynamic Backend Command Routing ---
  // Maps the selected UI loader to your new specific Rust commands
  const getBackendCommand = ():
    | "get_forge_versions"
    | "get_fabric_versions"
    | "get_vanilla_versions" => {
    switch (activeLoader) {
      case "forge":
        return "get_forge_versions";
      case "fabric":
        return "get_fabric_versions";
      case "vanilla":
      default:
        return "get_vanilla_versions";
    }
  };

  const { data, isLoading, error } = useBackend({
    name: getBackendCommand(),
  });

  const { mutateAsync: downloadVersion } = useBackendMutation({
    name: "download_version",
  });

  // --- Event Listeners ---
  useEffect(() => {
    let unlistenProgress: () => void;
    let unlistenProgressBar: () => void;

    async function setupListeners() {
      unlistenProgress = await listen<string>("progress", (event) => {
        setStatusText(event.payload);
      });

      unlistenProgressBar = await listen<number>("progressBar", (event) => {
        setProgress(event.payload);
        if (event.payload >= 100) {
          setIsDone(true);
        }
      });
    }

    setupListeners();

    return () => {
      if (unlistenProgress) {
        unlistenProgress();
      }
      if (unlistenProgressBar) {
        unlistenProgressBar();
      }
    };
  }, []);

  // --- State Synchronization ---
  useEffect(() => {
    setActiveVersion(null);
  }, [activeLoader]);

  useEffect(() => {
    // Safely filter out any undefined/null data or empty categories
    if (!(data && Array.isArray(data)) || data.length === 0) {
      return;
    }

    // Filter out categories that have empty version arrays (fixes UI showing empty lists)
    const validData = data.filter(
      (cat) => cat.versions && cat.versions.length > 0
    );
    if (validData.length === 0) {
      return;
    }

    const hasMajor = validData.some((v) => v.name === activeMajorVersion);
    const selectedMajor = hasMajor ? activeMajorVersion : validData[0].name;

    if (!hasMajor) {
      setActiveMajorVersion(selectedMajor);
    }

    const versions = validData.find((v) => selectedMajor === v.name)?.versions;
    const versionMatch = versions?.find((v) => v.id === activeVersion?.id);

    if (versions && !versionMatch) {
      setActiveVersion(versions[0] ?? null);
    }
  }, [data, activeMajorVersion, activeVersion, activeLoader]);

  // --- Actions ---
  const handleSelectLoader = (loader: LoaderType) => {
    setActiveLoader(loader);
    setStep(2);
  };

  const handleStartInstall = async () => {
    if (!activeVersion) {
      return;
    }

    setStep(3);
    setProgress(0);
    setIsDone(false);
    setStatusText("Initializing download...");

    await downloadVersion({
      name: instanceName,
      versionLoader: activeVersion,
      // name: instanceName || activeVersion.id
    });
  };

  const handleReset = () => {
    setStep(1);
    setProgress(null);
    setStatusText("");
    setIsDone(false);
    setInstanceName("");
  };

  return (
    <div className="flex h-full min-h-0 w-full items-center justify-center bg-background p-4">
      <div className="flex min-h-[450px] w-full max-w-2xl flex-col rounded-2xl border border-border/50 bg-secondary/20 p-6 shadow-xl backdrop-blur-md">
        {/* --- STEP 1: SELECT LOADER --- */}
        {step === 1 && (
          <div className="fade-in slide-in-from-bottom-4 flex h-full animate-in flex-col duration-300">
            <div className="mb-8 space-y-2 text-center">
              <div className="mx-auto mb-4 w-fit rounded-full bg-primary/20 p-3">
                <HugeiconsIcon
                  className="text-primary"
                  icon={PackageIcon}
                  size={32}
                />
              </div>
              <h2 className="font-bold text-2xl text-foreground tracking-tight">
                Choose Environment
              </h2>
              <p className="text-muted-foreground text-sm">
                Select the mod loader you want to install.
              </p>
            </div>

            <div className="mt-auto mb-auto grid grid-cols-1 gap-4 sm:grid-cols-3">
              {[
                { desc: "Standard Minecraft", id: "vanilla", label: "Vanilla" },
                { desc: "Lightweight & fast", id: "fabric", label: "Fabric" },
                { desc: "Heavy modpack support", id: "forge", label: "Forge" },
              ].map((loader) => (
                <button
                  className="group relative flex flex-col items-center justify-center rounded-xl border border-border/60 bg-background p-6 outline-none transition-all duration-200 hover:border-primary/50 hover:bg-primary/5 focus-visible:ring-2 focus-visible:ring-primary"
                  key={loader.id}
                  onClick={() => handleSelectLoader(loader.id as LoaderType)}
                >
                  <span className="mb-1 font-semibold text-lg capitalize transition-colors group-hover:text-primary">
                    {loader.label}
                  </span>
                  <span className="text-center text-muted-foreground text-xs">
                    {loader.desc}
                  </span>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* --- STEP 2: CONFIGURE INSTANCE --- */}
        {step === 2 && (
          <div className="fade-in slide-in-from-right-4 flex h-full animate-in flex-col duration-300">
            <div className="mb-6 flex items-center">
              <button
                className="-ml-2 rounded-lg p-2 text-muted-foreground transition-colors hover:bg-secondary/50 hover:text-foreground"
                onClick={() => setStep(1)}
              >
                <HugeiconsIcon icon={ArrowLeft02Icon} size={20} />
              </button>
              <h2 className="ml-2 font-bold text-xl tracking-tight">
                Configure Instance
              </h2>
              <div className="ml-auto rounded-full bg-primary/10 px-3 py-1 font-bold text-primary text-xs uppercase tracking-wider">
                {activeLoader}
              </div>
            </div>

            <LoadingSwap
              className="flex min-h-0 flex-1 flex-col"
              isLoading={isLoading}
            >
              {error ? (
                <div className="flex flex-1 items-center justify-center">
                  <Empty>
                    <EmptyMedia variant="icon">
                      <HugeiconsIcon icon={Alert01Icon} size={24} />
                    </EmptyMedia>
                    <EmptyTitle>{errorText(error.code).title}</EmptyTitle>
                    <EmptyDescription>
                      {errorText(error.code).description}
                    </EmptyDescription>
                  </Empty>
                </div>
              ) : (
                <>
                  <div className="scrollbar-thin flex-1 space-y-6 overflow-y-auto pr-2 pb-4">
                    <div className="space-y-2">
                      <label className="font-bold text-muted-foreground text-xs uppercase tracking-wider">
                        Instance Name
                      </label>
                      <input
                        className="w-full rounded-xl border border-border/80 bg-background px-4 py-2.5 font-medium text-foreground text-sm transition-colors placeholder:text-muted-foreground/50 focus:border-primary/60 focus:outline-none"
                        onChange={(e) => setInstanceName(e.target.value)}
                        placeholder={`e.g. My ${activeLoader.charAt(0).toUpperCase() + activeLoader.slice(1)} World`}
                        type="text"
                        value={instanceName}
                      />
                    </div>

                    <div className="space-y-2">
                      <label className="font-bold text-muted-foreground text-xs uppercase tracking-wider">
                        Major Version
                      </label>
                      <div className="flex flex-wrap gap-2">
                        {data
                          ?.filter(
                            (cat) => cat.versions && cat.versions.length > 0
                          )
                          .map((v: VersionCategory) => {
                            const isActive = activeMajorVersion === v.name;
                            return (
                              <button
                                className={`rounded-lg px-3.5 py-1.5 font-medium text-xs transition-all ${
                                  isActive
                                    ? "bg-primary text-primary-foreground shadow-md"
                                    : "border border-border/60 bg-background text-muted-foreground hover:border-primary/40 hover:text-foreground"
                                }`}
                                key={v.name}
                                onClick={() => setActiveMajorVersion(v.name)}
                              >
                                {v.name}
                              </button>
                            );
                          })}
                      </div>
                    </div>

                    <div className="space-y-2">
                      <label className="font-bold text-muted-foreground text-xs uppercase tracking-wider">
                        Specific Version
                      </label>
                      <Combobox
                        autoHighlight
                        items={
                          data?.find((v) => activeMajorVersion === v.name)
                            ?.versions || []
                        }
                        onValueChange={(val) => setActiveVersion(val)}
                        value={activeVersion}
                      >
                        <ComboboxInput
                          placeholder="Select a specific build..."
                          value={activeVersion?.id ?? ""}
                        />
                        <ComboboxContent>
                          <ComboboxEmpty>No versions found.</ComboboxEmpty>
                          <ComboboxList>
                            {(version) => (
                              <ComboboxItem key={version.id} value={version}>
                                {version.id}
                              </ComboboxItem>
                            )}
                          </ComboboxList>
                        </ComboboxContent>
                      </Combobox>
                    </div>
                  </div>

                  <div className="mt-auto shrink-0 border-border/40 border-t pt-4">
                    <ActionButton
                      action={handleStartInstall}
                      className="h-auto w-full gap-2 rounded-xl py-3 font-semibold text-sm"
                      disabled={
                        !activeVersion || (data && data.length === 0) || !!error
                      }
                    >
                      <HugeiconsIcon icon={Download01Icon} size={18} />
                      Start Installation
                    </ActionButton>
                  </div>
                </>
              )}
            </LoadingSwap>
          </div>
        )}

        {/* --- STEP 3: INSTALLING --- */}
        {step === 3 && (
          <div className="zoom-in-95 flex h-full animate-in flex-col items-center justify-center duration-300">
            <div className="mb-8 rounded-full border border-border/40 bg-secondary/40 p-4">
              {isDone ? (
                <HugeiconsIcon
                  className="spin-in-12 animate-in text-emerald-500 duration-500"
                  icon={CheckmarkBadge01Icon}
                  size={48}
                />
              ) : (
                <HugeiconsIcon
                  className="animate-spin text-primary"
                  icon={Settings01Icon}
                  size={48}
                />
              )}
            </div>

            <h2 className="mb-2 font-bold text-2xl text-foreground tracking-tight">
              {isDone ? "Installation Complete!" : "Installing..."}
            </h2>

            <p className="mb-8 max-w-[80%] truncate text-center text-muted-foreground text-sm">
              {isDone
                ? `Successfully installed ${instanceName || activeVersion?.id}`
                : statusText}
            </p>

            <div className="mb-8 w-full max-w-md space-y-2">
              <div className="h-3.5 w-full overflow-hidden rounded-full border border-border/60 bg-background p-0.5 shadow-inner">
                <div
                  className={`h-full rounded-full transition-all duration-300 ease-out ${isDone ? "bg-emerald-500" : "bg-primary"}`}
                  style={{
                    width: `${Math.min(Math.max(Number(progress) || 0, 0), 100)}%`,
                  }}
                />
              </div>
              <div className="text-right font-bold text-muted-foreground text-xs">
                {String(progress ?? 0)}%
              </div>
            </div>

            {isDone && (
              <ActionButton
                action={handleReset}
                className="px-8"
                variant="secondary"
              >
                Finish & Go Back
              </ActionButton>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
