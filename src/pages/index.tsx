import { Alert01Icon } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { app } from "@tauri-apps/api";
import { ActionButton } from "@/components/ui/action-button";
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "@/components/ui/combobox";
import { Empty, EmptyTitle } from "@/components/ui/empty";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";
import { errorText } from "@/messages";
import { useConfig } from "@/stores/config";

const getPanoramaUrl = (version: string | null, face: number) => {
  if (!version) {
    return `https://minecraft.wiki/images/1.21_panorama_${face}.png`;
  }

  const match = version.match(/1\.(\d+)/);
  if (match) {
    const minor = Number.parseInt(match[1], 10);
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
      <div className="relative flex h-full flex-1 flex-col overflow-hidden rounded-xl bg-black">
        {/* Animated 3D Panorama */}
        <div className="pointer-events-none absolute inset-0 overflow-hidden">
          <div className="flex animate-background">
            {[0, 1, 2, 3].map((face) => (
              <img
                className="h-screen object-cover"
                key={face}
                src={getPanoramaUrl(version, face)}
              />
            ))}
          </div>
        </div>

        {/* Gradient Overlay */}
        <div className="pointer-events-none absolute inset-0 bg-gradient-to-br from-[#2a2a2a]/60 to-[#111]/90" />

        {/* Content */}
        <div className="relative z-10 flex flex-1 flex-col justify-end p-8">
          <div className="max-w-2xl">
            <h2 className="mb-4 font-black text-5xl drop-shadow-lg">
              Welcome to Falcon
            </h2>
            <p className="text-gray-300 text-xl drop-shadow">
              The most advanced launcher.
            </p>
          </div>
        </div>

        {/* Bottom Action Bar */}
        <div className="relative z-10 flex h-24 items-center justify-between border-[#333] border-t bg-[#232323] px-8 shadow-[0_-10px_30px_rgba(0,0,0,0.5)]">
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
    initialData: [],
    initialDataUpdatedAt: 0,
    name: "get_installed_versions",
  });

  if (error) {
    return (
      <Empty className="h-12 w-full flex-row justify-start gap-2 rounded-xl border border-destructive/20 bg-destructive/5 p-2">
        <HugeiconsIcon
          className="text-destructive"
          icon={Alert01Icon}
          size={20}
        />
        <EmptyTitle className="text-destructive text-sm">
          {errorText(error.code).title}
        </EmptyTitle>
      </Empty>
    );
  }

  return (
    <Combobox
      autoHighlight
      items={installedVersions}
      onValueChange={(version) => setVersion(version)}
      value={version}
    >
      <ComboboxInput
        className="h-12 w-full border-[#333] bg-[#1a1a1a] text-white"
        placeholder="Select a Version"
      />
      <ComboboxContent className="border-[#333] bg-[#1a1a1a] text-white">
        <ComboboxEmpty>No items found.</ComboboxEmpty>
        <ComboboxList>
          {(version) => (
            <ComboboxItem
              className="hover:bg-[#333]"
              key={version}
              value={version}
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
    args: {
      app,
      selectedVersion: version ?? "",
    },
    name: "play",
  });

  return (
    <ActionButton
      action={async () => {
        await mutateAsync();
      }}
      className="h-14 w-full font-bold text-2xl"
      disabled={version === null || profile === null}
    >
      PLAY
    </ActionButton>
  );
}
