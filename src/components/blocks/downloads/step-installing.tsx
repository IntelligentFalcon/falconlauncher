import {
    Alert02Icon,
    CheckmarkBadge01Icon,
    Settings01Icon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ActionButton } from "@/components/ui/action-button";
import { useBackendMutation } from "@/hooks/use-backend";
import type { VersionLoader } from "@/invokes";

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

export function StepInstalling({
                                   activeVersion,
                                   instanceName,
                                   onReset,
                               }: {
    activeVersion: VersionLoader;
    instanceName: string;
    onReset: () => void;
}) {
    const { t } = useTranslation();

    const [progress, setProgress] = useState<DownloadProgress | null>(null);
    const [isDone, setIsDone] = useState<boolean>(false);
    const [errorMessage, setErrorMessage] = useState<string | null>(null);

    const { mutateAsync: downloadVersionMutation } = useBackendMutation({
        name: "download_version",
    });

    const isDownloadingRef = useRef(false);

    const startDownload = useCallback(async () => {
        if (isDownloadingRef.current) return;
        isDownloadingRef.current = true;

        // Switch view back to installing immediately
        setErrorMessage(null);
        setIsDone(false);
        setProgress(null);

        try {
            await downloadVersionMutation({
                versionLoader: activeVersion,
                name: instanceName || activeVersion.id,
            });
        } catch (err: any) {
            let message = t("stepInstalling.defaultError");

            if (typeof err === "string") {
                message = err;
            } else if (err?.message) {
                message = err.message;
            } else if (err?.code) {
                message = `${err.code}: ${err.data || ""}`;
            } else if (typeof err === "object") {
                message = JSON.stringify(err);
            }

            setErrorMessage(message);
        } finally {
            isDownloadingRef.current = false;
        }
    }, [activeVersion, instanceName, downloadVersionMutation, t]);

    // Initial download on mount
    useEffect(() => {
        startDownload();
    }, [startDownload]);

    // Listen to Tauri progress events
    useEffect(() => {
        const unlistenPromise = listen<DownloadProgress>(
            "download-progress",
            (event) => {
                const payload = event.payload;
                setProgress(payload);

                if (payload.global_percentage >= 100 || payload.stage === "done") {
                    setIsDone(true);
                }
            }
        );

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    }, []);

    const percentage = Math.min(
        Math.max(Number(progress?.global_percentage) || 0, 0),
        100
    );

    // --- ERROR SCREEN ---
    if (errorMessage) {
        return (
            <div className="zoom-in-95 flex h-full animate-in flex-col items-center justify-center duration-300">
                <div className="mb-8 rounded-full border border-destructive/40 bg-destructive/10 p-4">
                    <HugeiconsIcon
                        className="text-destructive"
                        icon={Alert02Icon}
                        size={48}
                    />
                </div>

                <h2 className="mb-2 font-bold text-2xl text-foreground tracking-tight">
                    {t("stepInstalling.errorTitle")}
                </h2>

                <p className="mb-8 max-w-md text-center text-muted-foreground text-sm">
                    {errorMessage}
                </p>

                <div className="flex items-center gap-3">
                    <ActionButton
                        action={async () => onReset()}
                        className="px-6"
                        variant="outline"
                    >
                        {t("stepInstalling.cancelButton")}
                    </ActionButton>

                    {/* Do NOT await the long-running startDownload inside the button handler */}
                    <ActionButton
                        action={() => {
                            setErrorMessage(null);
                            setIsDone(false);
                            setProgress(null);
                            setTimeout(() => {
                                startDownload();
                            }, 50);
                        }}
                        className="px-6"
                        variant="default"
                    >
                        {t("stepInstalling.retryButton")}
                    </ActionButton>
                </div>
            </div>
        );
    }

    // --- INSTALLING / SUCCESS SCREEN ---
    return (
        <div className="zoom-in-95 flex h-full animate-in flex-col items-center justify-center duration-300">
            <div className="mb-8 rounded-full border border-border/40 bg-secondary/40 p-4">
                {isDone ? (
                    <HugeiconsIcon
                        className="spin-in-12 animate-in text-emerald-500 duration-500"
                        icon={CheckmarkBadge01Icon}
                        size={48}
                    />
                ) : (
                    <HugeiconsIcon
                        className="animate-spin text-primary"
                        icon={Settings01Icon}
                        size={48}
                    />
                )}
            </div>

            <h2 className="mb-2 font-bold text-2xl text-foreground tracking-tight">
                {isDone
                    ? t("stepInstalling.completeTitle")
                    : t("stepInstalling.installingTitle")}
            </h2>

            <p className="mb-4 max-w-[85%] truncate text-center text-muted-foreground text-sm">
                {isDone
                    ? t("stepInstalling.successMessage", {
                        name: instanceName || activeVersion.id,
                    })
                    : progress?.stage_name || t("stepInstalling.initializing")}
                {!isDone && progress && progress.total_files > 1 && (
                    <span className="ml-1.5 opacity-70">
            ({progress.current_file}/{progress.total_files})
          </span>
                )}
            </p>

            <div className="mb-8 w-full max-w-md space-y-2">
                <div className="h-3.5 w-full overflow-hidden rounded-full border border-border/60 bg-background p-0.5 shadow-inner">
                    <div
                        className={`h-full rounded-full transition-all duration-300 ease-out ${
                            isDone ? "bg-emerald-500" : "bg-primary"
                        }`}
                        style={{ width: `${percentage}%` }}
                    />
                </div>

                <div className="flex items-center justify-between text-xs">
          <span className="max-w-[70%] truncate font-mono text-muted-foreground/80">
            {!isDone && progress?.file_name ? progress.file_name : ""}
          </span>
                    <span className="font-bold text-muted-foreground">
            {percentage.toFixed(0)}%
          </span>
                </div>
            </div>

            {isDone && (
                <ActionButton
                    action={async () => onReset()}
                    className="px-8"
                    variant="secondary"
                >
                    {t("stepInstalling.finishButton")}
                </ActionButton>
            )}
        </div>
    );
}