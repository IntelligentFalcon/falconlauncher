import {
    CheckmarkBadge01Icon,
    Settings01Icon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next"; // <-- Import added
import { ActionButton } from "@/components/ui/action-button";
import type { VersionLoader } from "@/invokes";

export function StepInstalling({
                                   activeVersion,
                                   instanceName,
                                   onReset,
                               }: {
    activeVersion: VersionLoader;
    instanceName: string;
    onReset: () => void;
}) {
    const { t } = useTranslation(); // <-- Initialize translation hook

    const [progress, setProgress] = useState<number | null>(0);
    // Set default to null so the translation hook handles the initial string dynamically
    const [statusText, setStatusText] = useState<string | null>(null);
    const [isDone, setIsDone] = useState<boolean>(false);

    useEffect(() => {
        const unlistenProgressPromise = listen<string>("progress", (event) => {
            setStatusText(event.payload);
        });

        const unlistenProgressBarPromise = listen<number>(
            "progressBar",
            (event) => {
                setProgress(event.payload);
                if (event.payload >= 100) {
                    setIsDone(true);
                }
            }
        );

        return () => {
            unlistenProgressPromise.then((unlisten) => unlisten());
            unlistenProgressBarPromise.then((unlisten) => unlisten());
        };
    }, []);

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
                {/* <-- Translated Titles */}
                {isDone ? t("stepInstalling.completeTitle") : t("stepInstalling.installingTitle")}
            </h2>

            <p className="mb-8 max-w-[80%] truncate text-center text-muted-foreground text-sm">
                {isDone
                    // <-- Translated Success Message with Dynamic Name
                    ? t("stepInstalling.successMessage", { name: instanceName || activeVersion.id })
                    // <-- Translated Fallback Initializing State
                    : (statusText || t("stepInstalling.initializing"))}
            </p>

            <div className="mb-8 w-full max-w-md space-y-2">
                <div className="h-3.5 w-full overflow-hidden rounded-full border border-border/60 bg-background p-0.5 shadow-inner">
                    <div
                        className={`h-full rounded-full transition-all duration-300 ease-out ${isDone ? "bg-emerald-500" : "bg-primary"}`}
                        style={{
                            width: `${Math.min(Math.max(Number(progress) || 0, 0), 100)}%`,
                        }}
                    />
                </div>
                <div className="text-right font-bold text-muted-foreground text-xs">
                    {String(progress ?? 0)}%
                </div>
            </div>

            {isDone && (
                <ActionButton
                    action={async () => onReset()}
                    className="px-8"
                    variant="secondary"
                >
                    {t("stepInstalling.finishButton")} {/* <-- Translated */}
                </ActionButton>
            )}
        </div>
    );
}