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
import {
  Empty,
  EmptyTitle,
} from '@/components/ui/empty';
import { errorText } from '@/messages';
import { Alert01Icon } from '@hugeicons/core-free-icons';

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

const getPanoramaUrl = (version: string | null, face: number) => {
  if (!version)
    return `https://minecraft.wiki/images/1.21_panorama_${face}.png`;

  const match = version.match(/1\.(\d+)/);
  if (match) {
    const minor = parseInt(match[1], 10);
    if (minor >= 26) {
      return `https://minecraft.wiki/images/EDU_26.30_panorama_${face}.png`;
    }
    if (minor < 14) {
      return `https://minecraft.wiki/images/Panorama_${face}_JE1.png`;
    }
    return `https://minecraft.wiki/images/1.${minor}_panorama_${face}.png`;
  }

  return `https://minecraft.wiki/images/1.21_panorama_${face}.png`;
};

export default function IndexPage() {
  const version = useConfig((state) => state.version);

  return (
    <div className="h-full">
      {/* Main Content Area */}
      <div className="flex-1 h-full flex flex-col relative bg-black overflow-hidden rounded-xl">
        {/* Animated 3D Panorama */}
        <div className="absolute inset-0 overflow-hidden pointer-events-none">
          <div className="flex animate-background">
            {[0, 1, 2, 3].map((face) => (
              <img
                key={face}
                src={getPanoramaUrl(version, face)}
                className="h-screen object-cover"
              />
            ))}
          </div>
        </div>

        {/* Gradient Overlay */}
        <div className="absolute inset-0 bg-gradient-to-br from-[#2a2a2a]/60 to-[#111]/90 pointer-events-none"></div>

        {/* Content */}
        <div className="flex-1 p-8 relative z-10 flex flex-col justify-end">
          <div className="max-w-2xl">
            <h2 className="text-5xl font-black mb-4 drop-shadow-lg">
              Welcome to Falcon
            </h2>
            <p className="text-xl text-gray-300 drop-shadow">
              The most advanced launcher.
            </p>
          </div>
        </div>

        {/* Bottom Action Bar */}
        <div className="relative z-10 h-24 bg-[#232323] border-t border-[#333] flex items-center justify-between px-8 shadow-[0_-10px_30px_rgba(0,0,0,0.5)]">
          <div className="w-64">
            <VersionSelect />
          </div>

          <div className="w-64">
            <PlayButton />
          </div>
        </div>
      </div>
    </div>
  );
}

function VersionSelect() {
  const { version, setVersion } = useConfig();

  const { data: installedVersions, error } = useBackend({
    name: 'get_installed_versions',
    initialData: [],
    initialDataUpdatedAt: 0,
  });

  if (error) {
    return (
      <Empty className="p-2 border border-destructive/20 h-12 flex-row gap-2 rounded-xl bg-destructive/5 justify-start w-full">
        <HugeiconsIcon icon={Alert01Icon} size={20} className="text-destructive" />
        <EmptyTitle className="text-sm text-destructive">{errorText(error.code).title}</EmptyTitle>
      </Empty>
    );
  }

  return (
    <Combobox
      items={installedVersions}
      autoHighlight
      value={version}
      onValueChange={(version) => setVersion(version)}
    >
      <ComboboxInput
        placeholder="Select a Version"
        className="w-full bg-[#1a1a1a] border-[#333] text-white h-12"
      />
      <ComboboxContent className="bg-[#1a1a1a] border-[#333] text-white">
        <ComboboxEmpty>No items found.</ComboboxEmpty>
        <ComboboxList>
          {(version) => (
            <ComboboxItem
              key={version}
              value={version}
              className="hover:bg-[#333]"
            >
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
      className="w-full h-14 text-2xl font-bold "
    >
      PLAY
    </ActionButton>
  );
}
