import { useState } from "react";
import { StepConfigureInstance } from "@/components/blocks/downloads/step-configure-instance";
import { StepInstalling } from "@/components/blocks/downloads/step-installing";
import { StepSelectLoader } from "@/components/blocks/downloads/step-select-loader";
import { useBackendMutation } from "@/hooks/use-backend";
import type { VersionLoader } from "@/invokes";

// Using Hugeicons to match your other steps
import { HugeiconsIcon } from "@hugeicons/react";
import { RefreshIcon } from "@hugeicons/core-free-icons"; // Swap this if your specific refresh icon is named differently

export type LoaderType = "vanilla" | "fabric" | "forge";
export type WizardStep = 1 | 2 | 3;

export default function InstallerWizard() {
  const [step, setStep] = useState<WizardStep>(1);
  const [activeLoader, setActiveLoader] = useState<LoaderType>("vanilla");
  const [activeVersion, setActiveVersion] = useState<VersionLoader | null>(
      null
  );
  const [instanceName, setInstanceName] = useState<string>("");
  const [isRefreshing, setIsRefreshing] = useState(false);

  const { mutateAsync: downloadVersion } = useBackendMutation({
    name: "download_version",
  });

  const { mutateAsync: reloadManifest } = useBackendMutation({
    name: "reload_version_manifest",
  });

  const handleSelectLoader = (loader: LoaderType) => {
    setActiveLoader(loader);
    setStep(2);
  };

  const handleStartInstall = async (version: VersionLoader, name: string) => {
    setActiveVersion(version);
    setInstanceName(name);
    setStep(3);

    try {
      await downloadVersion({
        name,
        versionLoader: version,
      });
    } catch (error) {
      console.error("Installation failed:", error);
    }
  };

  const handleReset = () => {
    setStep(1);
    setActiveVersion(null);
    setInstanceName("");
  };

  const handleRefresh = async () => {
    try {
      setIsRefreshing(true);
      await reloadManifest();
    } catch (error) {
      console.error("Failed to reload manifest:", error);
    } finally {
      setIsRefreshing(false);
    }
  };

  return (
      <div className="flex h-full min-h-0 w-full items-center justify-center bg-background p-4">
        <div className="relative flex min-h-[450px] w-full max-w-2xl flex-col rounded-2xl border border-border/50 bg-secondary/20 p-6 shadow-xl backdrop-blur-md">

          {/* Only render on Step 1 so it doesn't clash with Step 2's header or Step 3's centered layout */}
          {step === 1 && (
              <button
                  onClick={handleRefresh}
                  disabled={isRefreshing}
                  className="absolute right-6 top-6 z-10 rounded-md p-2 text-muted-foreground transition-colors hover:bg-secondary hover:text-foreground disabled:opacity-50"
                  title="Reload Manifest"
              >
                <HugeiconsIcon
                    icon={RefreshIcon}
                    size={20}
                    className={isRefreshing ? "animate-spin text-primary" : ""}
                />
              </button>
          )}

          {step === 1 && <StepSelectLoader onSelect={handleSelectLoader} />}
          {step === 2 && (
              <StepConfigureInstance
                  activeLoader={activeLoader}
                  onBack={() => setStep(1)}
                  onStartInstall={handleStartInstall}
              />
          )}
          {step === 3 && activeVersion && (
              <StepInstalling
                  activeVersion={activeVersion}
                  instanceName={instanceName}
                  onReset={handleReset}
              />
          )}
        </div>
      </div>
  );
}