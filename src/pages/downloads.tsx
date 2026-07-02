import { ActionButton } from '@/components/ui/action-button';
import { LoadingSwap } from '@/components/ui/animated/swapper';
import { Button } from '@/components/ui/button';
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

export default function Downloads() {
  const [activeVersion, setActiveVersion] = useState<VersionLoader | null>(
      null,
  );
  const [activeMajorVersion, setActiveMajorVersion] = useState('');

  // States to keep track of download progress and text
  const [progress, setProgress] = useState<number | null>(null);
  const [statusText, setStatusText] = useState<string>('');

  const { data, isLoading } = useBackend({
    name: 'get_categorized_versions',
    args: {
      forge: false,
      fabric: false,
      liteLoader: false,
      neoForge: false,
    },
  });

  // Listen to Rust backend events when the component loads
  useEffect(() => {
    let unlistenProgress: () => void;
    let unlistenProgressBar: () => void;

    async function setupListeners() {
      // Listen to the 'progress' text event from Rust
      unlistenProgress = await listen<string>('progress', (event) => {
        setStatusText(event.payload);
      });

      // Listen to the 'progressBar' number event from Rust
      unlistenProgressBar = await listen<number>('progressBar', (event) => {
        setProgress(event.payload);
      });
    }

    setupListeners();

    // Clean up listeners when leaving this page to prevent performance issues
    return () => {
      if (unlistenProgress) unlistenProgress();
      if (unlistenProgressBar) unlistenProgressBar();
    };
  }, []);

  useEffect(() => {
    if (!data) return;
    if (activeMajorVersion.length === 0) {
      setActiveMajorVersion(data[0].name);
    }
    const versions = data.find((v) => activeMajorVersion === v.name)?.versions;
    const versionMatch = versions?.find((v) => v === activeVersion);
    if (versions && !versionMatch) {
      setActiveVersion(versions[0]);
    }
  }, [data, activeMajorVersion]);

  const { mutateAsync: downloadVersion } = useBackendMutation({
    name: 'download_version',
  });

  return (
      <div className="flex h-full">
        <div className="bg-secondary p-1 space-y-1 w-min overflow-y-auto rounded-2xl">
          {data?.map((v) => (
              <Button
                  onClick={() => setActiveMajorVersion(v.name)}
                  variant={activeMajorVersion === v.name ? 'default' : 'outline'}
                  className="w-full"
                  key={v.name}
              >
                {v.name}
              </Button>
          ))}
        </div>
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
                    // Initialize the progress bar states right when clicking install
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
              Install
            </ActionButton>

            {/* Progress bar UI layout */}
            {progress !== null && (
                <div className="mt-4 w-full space-y-2">
                  {/* Outer Track */}
                  <div className="w-full bg-muted border border-border/40 rounded-full h-3 overflow-hidden shadow-inner">
                    {/* Inner Fill */}
                    <div
                        className="bg-primary h-full rounded-full transition-all duration-300 ease-out"
                        // Added Number() conversion and template fallback to guarantee it sets a string percentage like "45%"
                        style={{ width: `${Math.min(Math.max(Number(progress) || 0, 0), 100)}%` }}
                    />
                  </div>

                  {/* Status information */}
                  <div className="flex justify-between text-xs text-muted-foreground px-1 font-medium tracking-wide">
                    <span className="truncate max-w-[75%]">{statusText}</span>
                    {/* Displaying string format just in case it's parsed weirdly */}
                    <span>{String(progress)}%</span>
                  </div>
                </div>
            )}
          </LoadingSwap>
        </div>
      </div>
  );
}