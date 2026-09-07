import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Alert01Icon, PlayIcon, RepairIcon } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { app } from "@tauri-apps/api";
import { listen } from "@tauri-apps/api/event";
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

export interface DownloadProgress {
    stage: string;
    stage_name: string;
    current_file: number;
    total_files: number;
    current_bytes: number;
    total_bytes: number;
    file_name: string;
    global_percentage: number;
    stage_percentage: number;
}

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
    const { t } = useTranslation();

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
                            {t("index.welcome")}
                        </h2>
                        <p className="text-gray-300 text-xl drop-shadow">
                            {t("index.subtitle")}
                        </p>
                    </div>
                </div>

                {/* Bottom Action Bar */}
                <div className="relative z-10 flex flex-col border-[#333] border-t bg-[#232323] px-8 py-4 shadow-[0_-10px_30px_rgba(0,0,0,0.5)]">
                    <div className="flex min-h-24 flex-wrap items-center justify-between gap-4">
                        <div className="w-full sm:w-64">
                            <VersionSelect runningProcesses={runningProcesses} />
                        </div>

                        <div className="w-full sm:w-96">
                            <PlayButton runningProcesses={runningProcesses} />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

function VersionSelect({ runningProcesses }: { runningProcesses: string[] }) {
    const { version, setVersion } = useConfig();
    const { t } = useTranslation();

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
                placeholder={t("index.selectVersion")}
            />
            <ComboboxContent className="border-[#333] bg-[#1a1a1a] text-white">
                <ComboboxEmpty>{t("index.noItems")}</ComboboxEmpty>
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
    const { t } = useTranslation();

    const [progress, setProgress] = useState<DownloadProgress | null>(null);
    const [isDone, setIsDone] = useState<boolean>(false);

    useEffect(() => {
        const unlistenPromise = listen<DownloadProgress>(
            "download-progress",
            (event) => {
                const payload = event.payload;
                setProgress(payload);
                if (payload.global_percentage >= 100 || payload.stage === "done") {
                    setIsDone(true);
                    setTimeout(() => {
                        setProgress(null);
                        setIsDone(false);
                    }, 500);
                }
            }
        );
        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    }, []);

    const isRunning = version ? runningProcesses.includes(version) : false;
    const isRepairing = progress !== null && !isDone;

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

    const { mutateAsync: cancelDownloadMutation } = useBackendMutation({
        name: "cancel_download",
    });

    const handleAction = async () => {
        if (isRepairing) {
            await cancelDownloadMutation();
            setProgress(null);
            setIsDone(false);
        } else if (isRunning) {
            await killMutate();
        } else {
            await playMutate();
        }
    };

    const percentage = Math.min(
        Math.max(Number(progress?.global_percentage) || 0, 0),
        100
    );

    return (
        <>
            {isRepairing && (
                <div className="absolute bottom-full left-0 w-full px-8 pb-4">
                    <div className="mb-2 flex items-center justify-between text-xs">
                        <span className="max-w-[70%] truncate font-mono text-muted-foreground/80">
                            {progress?.stage_name || t("stepInstalling.initializing")}
                            {progress && progress.total_files > 1 && (
                                <span className="ml-1.5 opacity-70">
                                    ({progress.current_file}/{progress.total_files})
                                </span>
                            )}
                        </span>
                        <span className="font-bold text-muted-foreground">
                            {percentage.toFixed(0)}%
                        </span>
                    </div>
                    <div className="h-3.5 w-full overflow-hidden rounded-full border border-border/60 bg-background p-0.5 shadow-inner">
                        <div
                            className="h-full rounded-full transition-all duration-300 ease-out bg-primary"
                            style={{ width: `${percentage}%` }}
                        />
                    </div>
                </div>
            )}
            
            <div className="flex w-full flex-wrap items-center gap-3">
                <button
                    type="button"
                    disabled={isRunning || isRepairing}
                    aria-hidden={isRunning || isRepairing}
                    title={t("index.repairTooltip")}
                    onClick={() => setRepairMode((prev) => !prev)}
                    className={`flex h-14 w-14 sm:flex-none items-center justify-center rounded-xl border transition-all duration-300 ease-in-out ${
                        isRunning || isRepairing
                            ? "opacity-0 scale-95 pointer-events-none hidden"
                            : "opacity-100 scale-100"
                    } ${
                        repairMode
                            ? "border-amber-500 bg-amber-500/10 text-amber-500"
                            : "border-[#333] bg-[#1a1a1a] text-gray-400 hover:bg-[#333] hover:text-white"
                    }`}
                >
                    <HugeiconsIcon icon={RepairIcon} size={24} />
                </button>

                <ActionButton
                    action={handleAction}
                    className={`h-14 flex-1 font-bold text-2xl transition-all duration-300 ${
                        isRepairing
                            ? "bg-amber-500/20 text-amber-500 border border-amber-500/50 hover:bg-amber-600 hover:text-white hover:border-amber-600"
                            : isRunning
                            ? "bg-red-600/20 text-red-500 border border-red-500/50 hover:bg-red-600 hover:text-white hover:border-red-600"
                            : ""
                    }`}
                    disabled={version === null || profile === null}
                >
                    {isRepairing ? t("stepInstalling.abortButton") : isRunning ? t("index.stop") : t("index.play")}
                </ActionButton>
            </div>
        </>
    );
}