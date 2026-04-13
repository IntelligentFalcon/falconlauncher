import { useTranslation } from 'react-i18next';
import { useBackend, useBackendMutation } from '@/hooks/use-backend';
import { useQueryClient } from '@tanstack/react-query';
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from '@/components/ui/combobox';

import { app } from '@tauri-apps/api';
import { useConfig } from '@/stores/config';
import { ActionButton } from '@/components/ui/action-button';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { useRef } from 'react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Field, FieldGroup } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { HugeiconsIcon } from '@hugeicons/react';
import { BookXIcon, ConsoleIcon, UserPlus } from '@hugeicons/core-free-icons';

export default function IndexPage() {
  return (
    <div className="flex items-center justify-center">
      <div className="max-w-sm space-y-4">
        <h1 className="text-4xl text-center mb-0">Falcon</h1>
        <h2 className="text-2xl text-center mb-8">Launcher</h2>

        <VersionSelect />
        <PlayButton />
      </div>
    </div>
  );
}

function VersionSelect() {
  const { version, setVersion } = useConfig();

  const { data: installedVersions } = useBackend({
    name: 'get_installed_versions',
    initialData: [],
    initialDataUpdatedAt: 0,
  });

  return (
    <Combobox
      items={installedVersions}
      autoHighlight
      value={version}
      onValueChange={(version) => setVersion(version)}
    >
      <ComboboxInput placeholder="Select a Version" className="w-full" />
      <ComboboxContent>
        <ComboboxEmpty>No items found.</ComboboxEmpty>
        <ComboboxList>
          {(version) => (
            <ComboboxItem key={version} value={version}>
              {version}
            </ComboboxItem>
          )}
        </ComboboxList>
      </ComboboxContent>
    </Combobox>
  );
}

function PlayButton() {
  const version = useConfig((state) => state.version);
  const profile = useConfig((state) => state.profile);

  const { mutateAsync } = useBackendMutation({
    name: 'play',
    args: {
      app,
      selectedVersion: version ?? '',
    },
  });

  return (
    <ActionButton
      action={async () => {
        await mutateAsync();
      }}
      disabled={version === null || profile === null}
      className="w-full"
    >
      Play
    </ActionButton>
  );
}
