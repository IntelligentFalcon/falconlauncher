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
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

export default function Downloads() {
  // const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  // const [filters, setFilters] = useState({
  //   forge: false,
  //   fabric: false,
  //   neo_forge: false,
  //   lite_loader: false,
  // });
  const [activeVersion, setActiveVersion] = useState<VersionLoader | null>(
    null,
  );
  const [activeMajorVersion, setActiveMajorVersion] = useState('');

  const { data, isLoading } = useBackend({
    name: 'get_categorized_versions',
    args: {
      forge: false,
      fabric: true,
      liteLoader: false,
      neoForge: false,
    },
  });

  useEffect(() => {
    if (!data) return;
    if (activeMajorVersion.length === 0) {
      setActiveMajorVersion(data[0].name);
    }
    const versions = data.find((v) => activeMajorVersion === v.name)?.versions;
    if (versions && !activeVersion) {
      setActiveVersion(versions[0]);
    }
  }, [data, activeMajorVersion]);

  const { mutateAsync: downloadVersion } = useBackendMutation({
    name: 'download_version',
  });

  return (
    <div className="flex h-full">
      <div className="bg-secondary p-1 space-y-1 w-min overflow-y-auto">
        {data?.map((v) => (
          <Button
            onClick={() => setActiveMajorVersion(v.name)}
            variant={activeMajorVersion === v.name ? 'default' : 'outline'}
            className="w-full"
          >
            {v.name}
          </Button>
        ))}
      </div>
      <div className="flex-1">
        <LoadingSwap isLoading={isLoading} className="max-w-sm m-auto mt-8">
          <>
            <Combobox
              items={data?.find((v) => activeMajorVersion === v.name)?.versions}
              autoHighlight
              value={activeVersion}
              onValueChange={(val) => setActiveVersion(val)}
            >
              <ComboboxInput placeholder="Select a Version" />
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
                if (activeVersion)
                  await downloadVersion({
                    appHandle: app,
                    versionLoader: activeVersion,
                  });
              }}
              className="w-full mt-2"
            >
              Install
            </ActionButton>
          </>
        </LoadingSwap>
      </div>
    </div>
  );
}
