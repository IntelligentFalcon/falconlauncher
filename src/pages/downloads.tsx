import { useState } from "react";
import { StepConfigureInstance } from "@/components/blocks/downloads/step-configure-instance";
import { StepInstalling } from "@/components/blocks/downloads/step-installing";
import { StepSelectLoader } from "@/components/blocks/downloads/step-select-loader";
import { useBackendMutation } from "@/hooks/use-backend";
import type { VersionLoader } from "@/invokes";

export type LoaderType = "vanilla" | "fabric" | "forge";
export type WizardStep = 1 | 2 | 3;

export default function InstallerWizard() {
  const [step, setStep] = useState<WizardStep>(1);
  const [activeLoader, setActiveLoader] = useState<LoaderType>("vanilla");
  const [activeVersion, setActiveVersion] = useState<VersionLoader | null>(
    null
  );
  const [instanceName, setInstanceName] = useState<string>("");

  const { mutateAsync: downloadVersion } = useBackendMutation({
    name: "download_version",
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

  return (
    <div className="flex h-full min-h-0 w-full items-center justify-center bg-background p-4">
      <div className="flex min-h-[450px] w-full max-w-2xl flex-col rounded-2xl border border-border/50 bg-secondary/20 p-6 shadow-xl backdrop-blur-md">
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
