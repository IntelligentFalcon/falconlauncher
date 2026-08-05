import { ActionButton } from '@/components/ui/action-button';
import { LoadingSwap } from '@/components/ui/animated/swapper';
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from '@/components/ui/combobox';
import { useBackend, useBackendMutation } from '@/hooks/use-backend';
import { VersionLoader, VersionCategory } from '@/invokes';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import { HugeiconsIcon } from '@hugeicons/react';
import {
  ArrowLeft02Icon,
  Settings01Icon,
  Download01Icon,
  CheckmarkBadge01Icon,
  PackageIcon,
  Alert01Icon,
} from '@hugeicons/core-free-icons';
import {
  Empty,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty';
import { errorText } from '@/messages';

type LoaderType = 'vanilla' | 'fabric' | 'forge';
type WizardStep = 1 | 2 | 3;

export default function InstallerWizard() {
  // --- Wizard State ---
  const [step, setStep] = useState<WizardStep>(1);
  const [activeLoader, setActiveLoader] = useState<LoaderType>('vanilla');

  // --- Configuration State ---
  const [activeMajorVersion, setActiveMajorVersion] = useState<string>('');
  const [activeVersion, setActiveVersion] = useState<VersionLoader | null>(
    null,
  );
  const [instanceName, setInstanceName] = useState<string>('');

  // --- Installation State ---
  const [progress, setProgress] = useState<number | null>(null);
  const [statusText, setStatusText] = useState<string>('');
  const [isDone, setIsDone] = useState<boolean>(false);

  // --- Dynamic Backend Command Routing ---
  // Maps the selected UI loader to your new specific Rust commands
  const getBackendCommand = ():
    | 'get_forge_versions'
    | 'get_fabric_versions'
    | 'get_vanilla_versions' => {
    switch (activeLoader) {
      case 'forge':
        return 'get_forge_versions';
      case 'fabric':
        return 'get_fabric_versions';
      case 'vanilla':
      default:
        return 'get_vanilla_versions';
    }
  };

  const { data, isLoading, error } = useBackend({
    name: getBackendCommand(),
  });

  const { mutateAsync: downloadVersion } = useBackendMutation({
    name: 'download_version',
  });

  // --- Event Listeners ---
  useEffect(() => {
    let unlistenProgress: () => void;
    let unlistenProgressBar: () => void;

    async function setupListeners() {
      unlistenProgress = await listen<string>('progress', (event) => {
        setStatusText(event.payload);
      });

      unlistenProgressBar = await listen<number>('progressBar', (event) => {
        setProgress(event.payload);
        if (event.payload >= 100) {
          setIsDone(true);
        }
      });
    }

    setupListeners();

    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenProgressBar) unlistenProgressBar();
    };
  }, []);

  // --- State Synchronization ---
  useEffect(() => {
    setActiveVersion(null);
  }, [activeLoader]);

  useEffect(() => {
    // Safely filter out any undefined/null data or empty categories
    if (!data || !Array.isArray(data) || data.length === 0) return;

    // Filter out categories that have empty version arrays (fixes UI showing empty lists)
    const validData = data.filter(
      (cat) => cat.versions && cat.versions.length > 0,
    );
    if (validData.length === 0) return;

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
    if (!activeVersion) return;

    setStep(3);
    setProgress(0);
    setIsDone(false);
    setStatusText('Initializing download...');

    await downloadVersion({
      name: instanceName,
      versionLoader: activeVersion,
      // name: instanceName || activeVersion.id
    });
  };

  const handleReset = () => {
    setStep(1);
    setProgress(null);
    setStatusText('');
    setIsDone(false);
    setInstanceName('');
  };

  return (
    <div className="flex h-full w-full items-center justify-center p-4 min-h-0 bg-background">
      <div className="w-full max-w-2xl bg-secondary/20 backdrop-blur-md border border-border/50 rounded-2xl p-6 shadow-xl flex flex-col min-h-[450px]">
        {/* --- STEP 1: SELECT LOADER --- */}
        {step === 1 && (
          <div className="flex flex-col h-full animate-in fade-in slide-in-from-bottom-4 duration-300">
            <div className="mb-8 text-center space-y-2">
              <div className="mx-auto bg-primary/20 p-3 rounded-full w-fit mb-4">
                <HugeiconsIcon
                  icon={PackageIcon}
                  size={32}
                  className="text-primary"
                />
              </div>
              <h2 className="text-2xl font-bold tracking-tight text-foreground">
                Choose Environment
              </h2>
              <p className="text-muted-foreground text-sm">
                Select the mod loader you want to install.
              </p>
            </div>

            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4 mt-auto mb-auto">
              {[
                { id: 'vanilla', label: 'Vanilla', desc: 'Standard Minecraft' },
                { id: 'fabric', label: 'Fabric', desc: 'Lightweight & fast' },
                { id: 'forge', label: 'Forge', desc: 'Heavy modpack support' },
              ].map((loader) => (
                <button
                  key={loader.id}
                  onClick={() => handleSelectLoader(loader.id as LoaderType)}
                  className="group relative flex flex-col items-center justify-center p-6 bg-background border border-border/60 rounded-xl hover:border-primary/50 hover:bg-primary/5 transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-primary"
                >
                  <span className="text-lg font-semibold capitalize mb-1 group-hover:text-primary transition-colors">
                    {loader.label}
                  </span>
                  <span className="text-xs text-muted-foreground text-center">
                    {loader.desc}
                  </span>
                </button>
              ))}
            </div>
          </div>
        )}

        {/* --- STEP 2: CONFIGURE INSTANCE --- */}
        {step === 2 && (
          <div className="flex flex-col h-full animate-in fade-in slide-in-from-right-4 duration-300">
            <div className="flex items-center mb-6">
              <button
                onClick={() => setStep(1)}
                className="p-2 -ml-2 rounded-lg text-muted-foreground hover:text-foreground hover:bg-secondary/50 transition-colors"
              >
                <HugeiconsIcon icon={ArrowLeft02Icon} size={20} />
              </button>
              <h2 className="text-xl font-bold tracking-tight ml-2">
                Configure Instance
              </h2>
              <div className="ml-auto px-3 py-1 bg-primary/10 text-primary text-xs font-bold uppercase rounded-full tracking-wider">
                {activeLoader}
              </div>
            </div>

            <LoadingSwap
              isLoading={isLoading}
              className="flex-1 flex flex-col min-h-0"
            >
              {error ? (
                <div className="flex-1 flex items-center justify-center">
                  <Empty>
                    <EmptyMedia variant="icon">
                      <HugeiconsIcon icon={Alert01Icon} size={24} />
                    </EmptyMedia>
                    <EmptyTitle>
                      {errorText(error.code).title}
                    </EmptyTitle>
                    <EmptyDescription>
                      {errorText(error.code).description}
                    </EmptyDescription>
                  </Empty>
                </div>
              ) : (
                <>
                  <div className="space-y-6 overflow-y-auto pr-2 pb-4 scrollbar-thin flex-1">
                  <div className="space-y-2">
                    <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                    Instance Name
                  </label>
                  <input
                    type="text"
                    value={instanceName}
                    onChange={(e) => setInstanceName(e.target.value)}
                    placeholder={`e.g. My ${activeLoader.charAt(0).toUpperCase() + activeLoader.slice(1)} World`}
                    className="w-full bg-background border border-border/80 rounded-xl px-4 py-2.5 text-sm font-medium text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:border-primary/60 transition-colors"
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                    Major Version
                  </label>
                  <div className="flex flex-wrap gap-2">
                    {data
                      ?.filter((cat) => cat.versions && cat.versions.length > 0)
                      .map((v: VersionCategory) => {
                        const isActive = activeMajorVersion === v.name;
                        return (
                          <button
                            key={v.name}
                            onClick={() => setActiveMajorVersion(v.name)}
                            className={`px-3.5 py-1.5 rounded-lg text-xs font-medium transition-all ${
                              isActive
                                ? 'bg-primary text-primary-foreground shadow-md'
                                : 'bg-background border border-border/60 text-muted-foreground hover:border-primary/40 hover:text-foreground'
                            }`}
                          >
                            {v.name}
                          </button>
                        );
                      })}
                  </div>
                </div>

                <div className="space-y-2">
                  <label className="text-xs font-bold uppercase tracking-wider text-muted-foreground">
                    Specific Version
                  </label>
                  <Combobox
                    items={
                      data?.find((v) => activeMajorVersion === v.name)
                        ?.versions || []
                    }
                    autoHighlight
                    value={activeVersion}
                    onValueChange={(val) => setActiveVersion(val)}
                  >
                    <ComboboxInput
                      placeholder="Select a specific build..."
                      value={activeVersion?.id ?? ''}
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

              <div className="pt-4 mt-auto border-t border-border/40 shrink-0">
                <ActionButton
                  action={handleStartInstall}
                  disabled={!activeVersion || (data && data.length === 0) || !!error}
                  className="w-full py-3 h-auto text-sm font-semibold rounded-xl gap-2"
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
          <div className="flex flex-col h-full items-center justify-center animate-in zoom-in-95 duration-300">
            <div className="mb-8 p-4 bg-secondary/40 rounded-full border border-border/40">
              {isDone ? (
                <HugeiconsIcon
                  icon={CheckmarkBadge01Icon}
                  size={48}
                  className="text-emerald-500 animate-in spin-in-12 duration-500"
                />
              ) : (
                <HugeiconsIcon
                  icon={Settings01Icon}
                  size={48}
                  className="text-primary animate-spin"
                />
              )}
            </div>

            <h2 className="text-2xl font-bold tracking-tight text-foreground mb-2">
              {isDone ? 'Installation Complete!' : 'Installing...'}
            </h2>

            <p className="text-muted-foreground text-sm max-w-[80%] text-center truncate mb-8">
              {isDone
                ? `Successfully installed ${instanceName || activeVersion?.id}`
                : statusText}
            </p>

            <div className="w-full max-w-md space-y-2 mb-8">
              <div className="w-full bg-background border border-border/60 rounded-full h-3.5 overflow-hidden shadow-inner p-0.5">
                <div
                  className={`h-full rounded-full transition-all duration-300 ease-out ${isDone ? 'bg-emerald-500' : 'bg-primary'}`}
                  style={{
                    width: `${Math.min(Math.max(Number(progress) || 0, 0), 100)}%`,
                  }}
                />
              </div>
              <div className="text-right text-xs font-bold text-muted-foreground">
                {String(progress ?? 0)}%
              </div>
            </div>

            {isDone && (
              <ActionButton
                action={handleReset}
                variant="secondary"
                className="px-8"
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
