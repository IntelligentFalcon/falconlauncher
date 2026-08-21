import { useState } from "react";
import { useTranslation } from "react-i18next"; // <-- Added import
import { Alert01Icon, PlayIcon } from "@hugeicons/core-free-icons";
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
    const { t } = useTranslation(); // <-- Initialize translation hook

    // Poll the backend every 1.5 seconds to get the latest list of running processes
    const { data: runningProcesses = [] } = useBackend({
        name: "get_processes",
        refetchInterval: 1500,
    });

    return (
        <div className="h-full select-none">
            <div className="relative flex h-full flex-1 flex-col overflow-hidden rounded-xl bg-black">
                {/* Animated 3D Panorama */}
                <div className="pointer-events-none absolute inset-0 overflow-hidden">
                    <div className="flex animate-background">
                        {[0, 1, 2, 3].map((face) => (
                            <img
                                alt=""
                                className="pointer-events-none h-screen object-cover"
                                draggable={false}
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
                            {t("index.welcome")} {/* <-- Translated Welcome */}
                        </h2>
                        <p className="text-gray-300 text-xl drop-shadow">
                            {t("index.subtitle")} {/* <-- Translated Subtitle */}
                        </p>
                    </div>
                </div>

                {/* Bottom Action Bar */}
                <div className="relative z-10 flex min-h-24 flex-wrap items-center justify-between gap-4 border-[#333] border-t bg-[#232323] px-8 py-4 shadow-[0_-10px_30px_rgba(0,0,0,0.5)]">
                    <div className="w-full sm:w-64">
                        <VersionSelect runningProcesses={runningProcesses} />
                    </div>

                    <div className="w-full sm:w-96">
                        <PlayButton runningProcesses={runningProcesses} />
                    </div>
                </div>
            </div>
        </div>
    );
}

function VersionSelect({ runningProcesses }: { runningProcesses: string[] }) {
    const { version, setVersion } = useConfig();
    const { t } = useTranslation(); // <-- Initialize translation hook

    const { data: installedVersions, error } = useBackend({
        initialData: [],
        initialDataUpdatedAt: 0,
        name: "get_installed_versions",
    });

    if (error) {
        return (
            <Empty className="h-12 w-full flex-row justify-start gap-2 rounded-xl border border-destructive/20 bg-destructive/5 p-2">
                <HugeiconsIcon
                    className="pointer-events-none shrink-0 text-destructive"
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
                className="h-12 w-full select-text border-[#333] bg-[#1a1a1a] text-white"
                placeholder={t("index.selectVersion")} // <-- Translated Placeholder
            />
            <ComboboxContent className="border-[#333] bg-[#1a1a1a] text-white">
                <ComboboxEmpty>{t("index.noItems")}</ComboboxEmpty> {/* <-- Translated Empty State */}
                <ComboboxList>
                    {(itemVersion) => {
                        const isRunning = runningProcesses.includes(itemVersion);
                        return (
                            <ComboboxItem
                                className="hover:bg-[#333]"
                                key={itemVersion}
                                value={itemVersion}
                            >
                                <div className="flex w-full items-center justify-between">
                                    <span>{itemVersion}</span>
                                    {isRunning && (
                                        <HugeiconsIcon
                                            icon={PlayIcon}
                                            size={16}
                                            className="text-green-500 animate-pulse"
                                        />
                                    )}
                                </div>
                            </ComboboxItem>
                        );
                    }}
                </ComboboxList>
            </ComboboxContent>
        </Combobox>
    );
}

function PlayButton({ runningProcesses }: { runningProcesses: string[] }) {
    const version = useConfig((state) => state.version);
    const profile = useConfig((state) => state.profile);
    const [repairMode, setRepairMode] = useState(false);
    const { t } = useTranslation(); // <-- Initialize translation hook

    const isRunning = version ? runningProcesses.includes(version) : false;

    // Play mutation
    const { mutateAsync: playMutate } = useBackendMutation({
        args: {
            app,
            selectedVersion: version ?? "",
            repairMode: repairMode,
            profile: profile?.uuid,
        },
        name: "play",
    });

    // Kill mutation
    const { mutateAsync: killMutate } = useBackendMutation({
        args: {
            selectedProcess: version ?? "",
        },
        name: "kill_process",
    });

    return (
        <div className="flex w-full flex-wrap items-center gap-3">
            <button
                type="button"
                disabled={isRunning}
                aria-hidden={isRunning}
                title={t("index.repairTooltip")} // <-- Translated Tooltip
                onClick={() => setRepairMode((prev) => !prev)}
                className={`flex h-14 flex-1 sm:flex-none items-center justify-center rounded-xl border px-4 font-bold text-sm transition-all duration-300 ease-in-out ${
                    isRunning
                        ? "opacity-0 scale-95 pointer-events-none"
                        : "opacity-100 scale-100"
                } ${
                    repairMode
                        ? "border-amber-500 bg-amber-500/10 text-amber-500"
                        : "border-[#333] bg-[#1a1a1a] text-gray-400 hover:bg-[#333] hover:text-white"
                }`}
            >
                {/* Translating with a dynamic variable (ON/OFF) */}
                {t("index.repairMode", {
                    state: repairMode ? t("index.on") : t("index.off")
                })}
            </button>

            {/* Dynamic Action Button (PLAY / STOP) */}
            <ActionButton
                action={async () => {
                    if (isRunning) {
                        await killMutate();
                    } else {
                        await playMutate();
                    }
                }}
                className={`h-14 flex-1 font-bold text-2xl transition-all duration-300 ${
                    isRunning
                        ? "bg-red-600/20 text-red-500 border border-red-500/50 hover:bg-red-600 hover:text-white hover:border-red-600"
                        : ""
                }`}
                disabled={version === null || profile === null}
            >
                {isRunning ? t("index.stop") : t("index.play")} {/* <-- Translated Play/Stop */}
            </ActionButton>
        </div>
    );
}