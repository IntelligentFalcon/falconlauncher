import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { HugeiconsIcon } from "@hugeicons/react";
import {
    Alert02Icon,
    Cancel01Icon,
    Folder01Icon,
    RefreshIcon,
} from "@hugeicons/core-free-icons";
import { useBackendMutation } from "@/hooks/use-backend";
import { NativeChoice } from "@/invokes";

export type AppErrorPayload =
    | string
    | { code?: string; message?: string; data?: string }
    | Record<string, unknown>;

interface NativeOptionConfig {
    key: "java" | "openal" | "glfw";
    titleKey: string;
    descriptionKey: string;
    getCmd: string;
    setCmd: string;
    browseTitleKey: string;
    browseDirectoryOnly: boolean;
}

export function NativeSettings() {
    const { t } = useTranslation();
    const { mutateAsync: saveMutation } = useBackendMutation({ name: "save" });

    const [java, setJavaState] = useState<NativeChoice>({
        mode: "version_associated",
        path: "",
    });
    const [openal, setOpenalState] = useState<NativeChoice>({
        mode: "version_associated",
        path: "",
    });
    const [glfw, setGlfwState] = useState<NativeChoice>({
        mode: "version_associated",
        path: "",
    });

    const [isLoading, setIsLoading] = useState(true);
    const [activeError, setActiveError] = useState<{
        error: AppErrorPayload;
        retryAction: () => Promise<void>;
    } | null>(null);

    const { mutateAsync: setJavaMutation } = useBackendMutation({ name: "set_java" });
    const { mutateAsync: setOpenalMutation } = useBackendMutation({ name: "set_openal" });
    const { mutateAsync: setGlfwMutation } = useBackendMutation({ name: "set_glfw" });

    const parseError = (err: unknown): string => {
        if (typeof err === "string") return err;
        if (typeof err === "object" && err !== null) {
            const casted = err as { code?: string; message?: string; data?: string };
            if (casted.message) return casted.message;
            if (casted.code) return `${casted.code}: ${casted.data || ""}`;
            return JSON.stringify(err);
        }
        return t("nativeSettings.unknownError");
    };

    const fetchAllSettings = useCallback(async () => {
        setIsLoading(true);
        try {
            const [javaRes, openalRes, glfwRes] = await Promise.all([
                invoke<NativeChoice>("get_java"),
                invoke<NativeChoice>("get_openal"),
                invoke<NativeChoice>("get_glfw"),
            ]);

            setJavaState({
                mode: javaRes.mode || "version_associated",
                path: javaRes.path || "",
            });
            setOpenalState({
                mode: openalRes.mode || "version_associated",
                path: openalRes.path || "",
            });
            setGlfwState({
                mode: glfwRes.mode || "version_associated",
                path: glfwRes.path || "",
            });
        } catch (err) {
            setActiveError({
                error: err as AppErrorPayload,
                retryAction: fetchAllSettings,
            });
        } finally {
            setIsLoading(false);
        }
    }, [t]);

    useEffect(() => {
        fetchAllSettings();
    }, [fetchAllSettings]);

    const saveSetting = async (
        key: "java" | "openal" | "glfw",
        updated: NativeChoice
    ) => {
        const execute = async () => {
            if (key === "java") {
                setJavaState(updated);
                await setJavaMutation({ java: updated });
            } else if (key === "openal") {
                setOpenalState(updated);
                await setOpenalMutation({ openal: updated });
            } else if (key === "glfw") {
                setGlfwState(updated);
                await setGlfwMutation({ glfw: updated });
            }
            await saveMutation();
        };

        try {
            await execute();
        } catch (err) {
            setActiveError({
                error: err as AppErrorPayload,
                retryAction: execute,
            });
        }
    };

    const handleBrowse = async (
        key: "java" | "openal" | "glfw",
        browseTitleKey: string,
        directoryOnly: boolean
    ) => {
        try {
            const selected = await open({
                multiple: false,
                directory: directoryOnly,
                title: t(browseTitleKey),
            });

            if (selected && typeof selected === "string") {
                await saveSetting(key, { mode: "custom", path: selected });
            }
        } catch (err) {
            setActiveError({
                error: err as AppErrorPayload,
                retryAction: async () => handleBrowse(key, browseTitleKey, directoryOnly),
            });
        }
    };

    const sections: NativeOptionConfig[] = [
        {
            key: "java",
            titleKey: "nativeSettings.javaRuntimeTitle",
            descriptionKey: "nativeSettings.javaRuntimeDescription",
            getCmd: "get_java",
            setCmd: "set_java",
            browseTitleKey: "nativeSettings.selectJavaFolder",
            browseDirectoryOnly: true,
        },
        {
            key: "openal",
            titleKey: "nativeSettings.openalTitle",
            descriptionKey: "nativeSettings.openalDescription",
            getCmd: "get_openal",
            setCmd: "set_openal",
            browseTitleKey: "nativeSettings.selectOpenalFolder",
            browseDirectoryOnly: true,
        },
        {
            key: "glfw",
            titleKey: "nativeSettings.glfwTitle",
            descriptionKey: "nativeSettings.glfwDescription",
            getCmd: "get_glfw",
            setCmd: "set_glfw",
            browseTitleKey: "nativeSettings.selectGlfwFolder",
            browseDirectoryOnly: true,
        },
    ];

    const getStateByKey = (key: "java" | "openal" | "glfw"): NativeChoice => {
        if (key === "java") return java;
        if (key === "openal") return openal;
        return glfw;
    };

    if (isLoading) {
        return (
            <div className="flex h-32 items-center justify-center text-xs text-muted-foreground">
                {t("nativeSettings.loading")}
            </div>
        );
    }

    return (
        <div className="relative space-y-8">
            {sections.map((sec) => {
                const current = getStateByKey(sec.key);
                const isCustom = current.mode === "custom";

                return (
                    <div
                        key={sec.key}
                        className="space-y-4 border-b border-border/40 pb-6 last:border-0 last:pb-0"
                    >
                        <div>
                            <h3 className="font-medium text-base text-foreground">
                                {t(sec.titleKey)}
                            </h3>
                            <p className="text-muted-foreground text-sm">
                                {t(sec.descriptionKey)}
                            </p>
                        </div>

                        <div className="space-y-3">
                            <label className="flex cursor-pointer items-center gap-3 rounded-lg border border-border/50 bg-secondary/10 p-3 transition-colors hover:bg-secondary/20">
                                <input
                                    type="radio"
                                    name={`choice_${sec.key}`}
                                    checked={!isCustom}
                                    onChange={() =>
                                        saveSetting(sec.key, {
                                            mode: "version_associated",
                                            path: current.path,
                                        })
                                    }
                                    className="h-4 w-4 text-primary focus:ring-primary"
                                />
                                <div>
                                    <p className="font-medium text-sm text-foreground">
                                        {t("nativeSettings.versionAssociated")}
                                    </p>
                                    <p className="text-muted-foreground text-xs">
                                        {t("nativeSettings.versionAssociatedDesc")}
                                    </p>
                                </div>
                            </label>

                            <label className="flex cursor-pointer items-start gap-3 rounded-lg border border-border/50 bg-secondary/10 p-3 transition-colors hover:bg-secondary/20">
                                <input
                                    type="radio"
                                    name={`choice_${sec.key}`}
                                    checked={isCustom}
                                    onChange={() =>
                                        saveSetting(sec.key, {
                                            mode: "custom",
                                            path: current.path,
                                        })
                                    }
                                    className="mt-1 h-4 w-4 text-primary focus:ring-primary"
                                />
                                <div className="w-full space-y-3">
                                    <div>
                                        <p className="font-medium text-sm text-foreground">
                                            {t("nativeSettings.custom")}
                                        </p>
                                        <p className="text-muted-foreground text-xs">
                                            {t("nativeSettings.customDesc")}
                                        </p>
                                    </div>

                                    {isCustom && (
                                        <div className="flex w-full items-center gap-2 pt-1">
                                            <input
                                                type="text"
                                                value={current.path}
                                                onChange={(e) =>
                                                    saveSetting(sec.key, {
                                                        mode: "custom",
                                                        path: e.target.value,
                                                    })
                                                }
                                                placeholder={t("nativeSettings.folderPlaceholder")}
                                                className="h-9 flex-1 rounded-md border border-border/80 bg-background px-3 font-mono text-xs text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary"
                                            />
                                            <button
                                                type="button"
                                                onClick={() =>
                                                    handleBrowse(
                                                        sec.key,
                                                        sec.browseTitleKey,
                                                        sec.browseDirectoryOnly
                                                    )
                                                }
                                                className="flex h-9 items-center gap-1.5 rounded-md border border-border bg-secondary px-3 text-xs text-secondary-foreground transition-colors hover:bg-secondary/80"
                                            >
                                                <HugeiconsIcon icon={Folder01Icon} size={14} />
                                                <span>{t("nativeSettings.browse")}</span>
                                            </button>
                                        </div>
                                    )}
                                </div>
                            </label>
                        </div>
                    </div>
                );
            })}

            {/* Error Modal */}
            {activeError && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 p-4 backdrop-blur-sm animate-in fade-in duration-200">
                    <div className="w-full max-w-md space-y-4 rounded-xl border border-destructive/40 bg-card p-6 shadow-2xl">
                        <div className="flex items-start gap-3">
                            <div className="rounded-full border border-destructive/30 bg-destructive/10 p-2 text-destructive">
                                <HugeiconsIcon icon={Alert02Icon} size={24} />
                            </div>
                            <div className="flex-1">
                                <h3 className="font-semibold text-base text-foreground">
                                    {t("nativeSettings.errorTitle")}
                                </h3>
                                <p className="mt-1 break-all rounded border border-border/50 bg-secondary/30 p-2 font-mono text-xs text-muted-foreground">
                                    {parseError(activeError.error)}
                                </p>
                            </div>
                        </div>

                        <div className="flex items-center justify-end gap-2 pt-2">
                            <button
                                type="button"
                                onClick={() => setActiveError(null)}
                                className="flex items-center gap-1.5 rounded-md border border-border bg-secondary px-4 py-2 font-medium text-secondary-foreground text-xs transition-colors hover:bg-secondary/80"
                            >
                                <HugeiconsIcon icon={Cancel01Icon} size={14} />
                                <span>{t("nativeSettings.close")}</span>
                            </button>

                            <button
                                type="button"
                                onClick={async () => {
                                    const action = activeError.retryAction;
                                    setActiveError(null);
                                    await action();
                                }}
                                className="flex items-center gap-1.5 rounded-md bg-primary px-4 py-2 font-medium text-primary-foreground text-xs transition-colors hover:bg-primary/90"
                            >
                                <HugeiconsIcon icon={RefreshIcon} size={14} />
                                <span>{t("nativeSettings.retry")}</span>
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}