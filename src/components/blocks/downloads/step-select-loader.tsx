import { PackageIcon } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { useTranslation } from "react-i18next"; // <-- Import added
import type { LoaderType } from "@/pages/downloads";

export function StepSelectLoader({
                                     onSelect,
                                 }: {
    onSelect: (loader: LoaderType) => void;
}) {
    const { t } = useTranslation(); // <-- Initialize translation hook

    return (
        <div className="fade-in slide-in-from-bottom-4 flex h-full animate-in flex-col duration-300">
            <div className="mb-8 space-y-2 text-center">
                <div className="mx-auto mb-4 w-fit rounded-full bg-primary/20 p-3">
                    <HugeiconsIcon
                        className="text-primary"
                        icon={PackageIcon}
                        size={32}
                    />
                </div>
                <h2 className="font-bold text-2xl text-foreground tracking-tight">
                    {t("stepSelectLoader.title")} {/* <-- Translated */}
                </h2>
                <p className="text-muted-foreground text-sm">
                    {t("stepSelectLoader.subtitle")} {/* <-- Translated */}
                </p>
            </div>

            <div className="mt-auto mb-auto grid grid-cols-1 gap-4 sm:grid-cols-3">
                {(
                    [
                        {
                            desc: t("stepSelectLoader.vanillaDesc"),
                            id: "vanilla",
                            label: t("stepSelectLoader.vanillaLabel")
                        },
                        {
                            desc: t("stepSelectLoader.fabricDesc"),
                            id: "fabric",
                            label: t("stepSelectLoader.fabricLabel")
                        },
                        {
                            desc: t("stepSelectLoader.forgeDesc"),
                            id: "forge",
                            label: t("stepSelectLoader.forgeLabel")
                        },
                    ] as const
                ).map((loader) => (
                    <button
                        className="group relative flex flex-col items-center justify-center rounded-xl border border-border/60 bg-background p-6 outline-none transition-all duration-200 hover:border-primary/50 hover:bg-primary/5 focus-visible:ring-2 focus-visible:ring-primary"
                        key={loader.id}
                        onClick={() => onSelect(loader.id)}
                        type="button"
                    >
            <span className="mb-1 font-semibold text-lg capitalize transition-colors group-hover:text-primary">
              {loader.label}
            </span>
                        <span className="text-center text-muted-foreground text-xs">
              {loader.desc}
            </span>
                    </button>
                ))}
            </div>
        </div>
    );
}