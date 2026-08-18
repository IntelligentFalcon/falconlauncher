import { useState } from "react";
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

const MINECRAFT_MINOR_VERSION_REGEX = /1\.(\d+)/;

const getPanoramaUrl = (version: string | null, face: number) => {
  if (!version) {
    return `https://minecraft.wiki/images/1.21_panorama_${face}.png`;
  }

  const match = version.match(MINECRAFT_MINOR_VERSION_REGEX);
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
                      alt=""
                      className="h-screen object-cover"
                      height={1080}
                      key={face}
                      src={getPanoramaUrl(version, face)}
                      width={1920}
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

            {/* Increased width from w-64 to w-96 to fit both buttons comfortably */}
            <div className="w-96">
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
          onValueChange={(newVersion) => setVersion(newVersion)}
          value={version}
      >
        <ComboboxInput
            className="h-12 w-full border-[#333] bg-[#1a1a1a] text-white"
            placeholder="Select a Version"
        />
        <ComboboxContent className="border-[#333] bg-[#1a1a1a] text-white">
          <ComboboxEmpty>No items found.</ComboboxEmpty>
          <ComboboxList>
            {(itemVersion) => (
                <ComboboxItem
                    className="hover:bg-[#333]"
                    key={itemVersion}
                    value={itemVersion}
                >
                  {itemVersion}
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

  // Repair mode state
  const [repairMode, setRepairMode] = useState(false);

  const { mutateAsync } = useBackendMutation({
    args: {
      app,
      selectedVersion: version ?? "",
      repairMode: repairMode,
      profile: profile?.uuid,
    },
    name: "play",
  });

  return (
      <div className="flex w-full items-center gap-3">
        {/* Repair Mode Toggle */}
        <button
            type="button"
            title="Downloads required files if they're not installed/corrupted. this option is only recommended to use if the selected version crashes."
            onClick={() => setRepairMode((prev) => !prev)}
            className={`flex h-14 shrink-0 items-center justify-center rounded-xl border px-4 font-bold text-sm transition-colors ${
                repairMode
                    ? "border-amber-500 bg-amber-500/10 text-amber-500"
                    : "border-[#333] bg-[#1a1a1a] text-gray-400 hover:bg-[#333] hover:text-white"
            }`}
        >
          Repair Mode: {repairMode ? "ON" : "OFF"}
        </button>

        {/* Play Button */}
        <ActionButton
            action={async () => {
              await mutateAsync();
            }}
            className="h-14 flex-1 font-bold text-2xl"
            disabled={version === null || profile === null}
        >
          PLAY
        </ActionButton>
      </div>
  );
}