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
import { VersionLoader } from '@/invokes';
import { app } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';

type LoaderType = 'vanilla' | 'fabric' | 'forge';

export default function Downloads() {
  const [activeLoader, setActiveLoader] = useState<LoaderType>('vanilla');
  const [activeVersion, setActiveVersion] = useState<VersionLoader | null>(null);
  const [activeMajorVersion, setActiveMajorVersion] = useState('');

  // States to keep track of download progress and text
  const [progress, setProgress] = useState<number | null>(null);
  const [statusText, setStatusText] = useState<string>('');

  // Dynamically pass parameters based on the active tab selection
  const { data, isLoading } = useBackend({
    name: 'get_categorized_versions',
    args: {
      forge: activeLoader === 'forge',
      fabric: activeLoader === 'fabric',
      liteLoader: false,
      neoForge: false,
    },
  });

  // Listen to Rust backend events when component mounts
  useEffect(() => {
    let unlistenProgress: () => void;
    let unlistenProgressBar: () => void;

    async function setupListeners() {
      unlistenProgress = await listen<string>('progress', (event) => {
        setStatusText(event.payload);
      });

      unlistenProgressBar = await listen<number>('progressBar', (event) => {
        setProgress(event.payload);
      });
    }

    setupListeners();

    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenProgressBar) unlistenProgressBar();
    };
  }, []);

  // Sync state safely when data changes or category tab is clicked
  useEffect(() => {
    if (!data || data.length === 0) return;

    // Check if current major version selection exists in the returned data
    const hasMajor = data.some((v) => v.name === activeMajorVersion);
    const selectedMajor = hasMajor ? activeMajorVersion : data[0].name;

    if (!hasMajor) {
      setActiveMajorVersion(selectedMajor);
    }

    // Find versions list for the selected major version
    const versions = data.find((v) => selectedMajor === v.name)?.versions;

    // Safely compare version IDs to prevent null references or stale state
    const versionMatch = versions?.find((v) => v.id === activeVersion?.id);

    if (versions && !versionMatch) {
      setActiveVersion(versions[0] ?? null);
    }
  }, [data, activeMajorVersion, activeLoader]);

  const { mutateAsync: downloadVersion } = useBackendMutation({
    name: 'download_version',
  });

  const categories: { id: LoaderType; label: string }[] = [
    { id: 'vanilla', label: 'Vanilla' },
    { id: 'fabric', label: 'Fabric' },
    { id: 'forge', label: 'Forge' },
  ];

  return (
      <div className="flex h-full space-x-6">
        {/* Sidebar with Categories */}
        <aside className="w-60 bg-secondary/30 backdrop-blur-md p-2.5 flex flex-col rounded-2xl border border-border/40 shrink-0 shadow-sm">
          {/* Mod Loader Category Selector Segment */}
          <div className="mb-3 space-y-1">
          <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/80 px-2">
            Mod Loader
          </span>
            <div className="grid grid-cols-3 gap-1 bg-background/50 p-1 rounded-xl border border-border/50">
              {categories.map((cat) => (
                  <button
                      key={cat.id}
                      onClick={() => setActiveLoader(cat.id)}
                      className={`py-1.5 text-xs font-semibold rounded-lg transition-all text-center ${
                          activeLoader === cat.id
                              ? 'bg-primary text-primary-foreground shadow-sm'
                              : 'text-muted-foreground hover:text-foreground hover:bg-secondary/50'
                      }`}
                  >
                    {cat.label}
                  </button>
              ))}
            </div>
          </div>

          {/* Major Versions List Header */}
          <div className="px-2 mb-1">
          <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/80">
            Major Versions
          </span>
          </div>

          {/* Major Version Options */}
          <div className="space-y-1 overflow-y-auto pr-1 flex-1 scrollbar-thin scrollbar-thumb-muted-foreground/20">
            {data?.map((v) => {
              const isActive = activeMajorVersion === v.name;
              return (
                  <button
                      key={v.name}
                      onClick={() => setActiveMajorVersion(v.name)}
                      className={`w-full flex items-center justify-between px-3.5 py-2 rounded-xl text-xs font-medium transition-all duration-200 outline-none ${
                          isActive
                              ? 'bg-primary text-primary-foreground shadow-md shadow-primary/20 font-semibold'
                              : 'text-muted-foreground hover:bg-secondary/80 hover:text-foreground'
                      }`}
                  >
                    <span>{v.name}</span>
                    {isActive && (
                        <span className="h-1.5 w-1.5 rounded-full bg-primary-foreground animate-pulse" />
                    )}
                  </button>
              );
            })}
          </div>
        </aside>

        {/* Main Content Area */}
        <div className="flex-1">
          <LoadingSwap isLoading={isLoading} className="max-w-sm m-auto mt-8">
            <Combobox
                items={data?.find((v) => activeMajorVersion === v.name)?.versions}
                autoHighlight
                value={activeVersion}
                onValueChange={(val) => setActiveVersion(val)}
            >
              <ComboboxInput
                  placeholder="Select a Version"
                  value={activeVersion?.id ?? ''}
              />
              <ComboboxContent>
                <ComboboxEmpty>No items found.</ComboboxEmpty>
                <ComboboxList>
                  {(version) => (
                      <ComboboxItem key={version.id} value={version}>
                        {version.id}
                      </ComboboxItem>
                  )}
                </ComboboxList>
              </ComboboxContent>
            </Combobox>

            <ActionButton
                action={async () => {
                  if (activeVersion) {
                    setProgress(0);
                    setStatusText('Initializing download...');

                    await downloadVersion({
                      appHandle: app,
                      versionLoader: activeVersion,
                    });
                  }
                }}
                className="w-full mt-2"
            >
              Install {activeLoader !== 'vanilla' ? `(${activeLoader.toUpperCase()})` : ''}
            </ActionButton>

            {/* Progress bar UI layout */}
            {progress !== null && (
                <div className="mt-4 w-full space-y-2">
                  <div className="w-full bg-muted border border-border/40 rounded-full h-3 overflow-hidden shadow-inner">
                    <div
                        className="bg-primary h-full rounded-full transition-all duration-300 ease-out"
                        style={{
                          width: `${Math.min(
                              Math.max(Number(progress) || 0, 0),
                              100
                          )}%`,
                        }}
                    />
                  </div>

                  <div className="flex justify-between text-xs text-muted-foreground px-1 font-medium tracking-wide">
                    <span className="truncate max-w-[75%]">{statusText}</span>
                    <span>{String(progress)}%</span>
                  </div>
                </div>
            )}
          </LoadingSwap>
        </div>
      </div>
  );
}