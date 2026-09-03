import {
  Alert01Icon,
  ArrowLeft02Icon,
  Download01Icon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { useState } from "react";
import { ActionButton } from "@/components/ui/action-button";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "@/components/ui/combobox";
import {
  Empty,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { useBackend } from "@/hooks/use-backend";
import type { VersionCategory, VersionLoader } from "@/invokes";
import { errorText } from "@/messages";
import type { LoaderType } from "@/pages/downloads";
import {useTranslation} from "react-i18next";

export function StepConfigureInstance({
  activeLoader,
  onBack,
  onStartInstall,
}: {
  activeLoader: LoaderType;
  onBack: () => void;
  onStartInstall: (version: VersionLoader, name: string) => void;
}) {
  const { t } = useTranslation();
  const [localActiveMajorVersion, setLocalActiveMajorVersion] =
    useState<string>("");
  const [localActiveVersion, setLocalActiveVersion] =
    useState<VersionLoader | null>(null);
  const [instanceName, setInstanceName] = useState<string>("");

  const getBackendCommand = ():
    | "get_forge_versions"
    | "get_fabric_versions"
    | "get_vanilla_versions" => {
    switch (activeLoader) {
      case "forge":
        return "get_forge_versions";
      case "fabric":
        return "get_fabric_versions";
      default:
        return "get_vanilla_versions";
    }
  };

  const { data, isLoading, error } = useBackend({ name: getBackendCommand() });

  const validData = Array.isArray(data)
    ? data.filter((cat) => cat.versions && cat.versions.length > 0)
    : [];

  const hasMajor = validData.some((v) => v.name === localActiveMajorVersion);
  const activeMajorVersion = hasMajor
    ? localActiveMajorVersion
    : validData[0]?.name || "";

  const versions =
    validData.find((v) => activeMajorVersion === v.name)?.versions || [];
  const versionMatch = versions.find((v) => v.id === localActiveVersion?.id);
  const activeVersion = versionMatch || versions[0] || null;

  return (
    <div className="fade-in slide-in-from-right-4 flex h-full animate-in flex-col duration-300">
      <div className="mb-6 flex items-center">
        <button
          className="-ml-2 rounded-lg p-2 text-muted-foreground transition-colors hover:bg-secondary/50 hover:text-foreground"
          onClick={onBack}
          type="button"
        >
          <HugeiconsIcon icon={ArrowLeft02Icon} size={20} />
        </button>
        <h2 className="ml-2 font-bold text-xl tracking-tight">
          {t("stepConfigureInstance.title")}
        </h2>
        <div className="ml-auto rounded-full bg-primary/10 px-3 py-1 font-bold text-primary text-xs uppercase tracking-wider">
          {activeLoader}
        </div>
      </div>

      <LoadingSwap
        className="flex min-h-0 flex-1 flex-col"
        isLoading={isLoading}
      >
        {error ? (
          <div className="flex flex-1 items-center justify-center">
            <Empty>
              <EmptyMedia variant="icon">
                <HugeiconsIcon icon={Alert01Icon} size={24} />
              </EmptyMedia>
              <EmptyTitle>{errorText(error.code).title}</EmptyTitle>
              <EmptyDescription>
                {errorText(error.code).description}
              </EmptyDescription>
            </Empty>
          </div>
        ) : (
          <>
            <div className="scrollbar-thin flex-1 space-y-6 overflow-y-auto pr-2 pb-4">
              <div className="space-y-2">
                <label
                  className="font-bold text-muted-foreground text-xs uppercase tracking-wider"
                  htmlFor="instance-name"
                >
                  {t("stepConfigureInstance.instanceName")}
                </label>
                <input
                  className="w-full rounded-xl border border-border/80 bg-background px-4 py-2.5 font-medium text-foreground text-sm transition-colors placeholder:text-muted-foreground/50 focus:border-primary/60 focus:outline-none"
                  id="instance-name"
                  onChange={(e) => setInstanceName(e.target.value)}
                  placeholder={t("stepConfigureInstance.instancePlaceholder")}
                  type="text"
                  value={instanceName}
                />
              </div>

              <div className="space-y-2">
                <span className="font-bold text-muted-foreground text-xs uppercase tracking-wider">
                  {t("stepConfigureInstance.majorVersion")}
                </span>
                <div className="flex flex-wrap gap-2">
                  {data
                    ?.filter((cat) => cat.versions && cat.versions.length > 0)
                    .map((v: VersionCategory) => {
                      const isActive = activeMajorVersion === v.name;
                      return (
                        <button
                          className={`rounded-lg px-3.5 py-1.5 font-medium text-xs transition-all ${
                            isActive
                              ? "bg-primary text-primary-foreground shadow-md"
                              : "border border-border/60 bg-background text-muted-foreground hover:border-primary/40 hover:text-foreground"
                          }`}
                          key={v.name}
                          onClick={() => {
                            setLocalActiveMajorVersion(v.name);
                            setLocalActiveVersion(null);
                          }}
                          type="button"
                        >
                          {v.name}
                        </button>
                      );
                    })}
                </div>
              </div>

              <div className="space-y-2">
                <span className="font-bold text-muted-foreground text-xs uppercase tracking-wider">
                  {t("stepConfigureInstance.selectVersion")}
                </span>
                <Combobox
                  autoHighlight
                  items={
                    data?.find((v) => activeMajorVersion === v.name)
                      ?.versions || []
                  }
                  onValueChange={(val) => setLocalActiveVersion(val)}
                  value={activeVersion}
                >
                  <ComboboxInput
                    placeholder="Select a specific build..."
                    value={activeVersion?.id ?? ""}
                  />
                  <ComboboxContent>
                    <ComboboxEmpty>No versions found.</ComboboxEmpty>
                    <ComboboxList>
                      {(version) => (
                        <ComboboxItem key={version.id} value={version}>
                          {version.id}
                        </ComboboxItem>
                      )}
                    </ComboboxList>
                  </ComboboxContent>
                </Combobox>
              </div>
            </div>

            <div className="mt-auto shrink-0 border-border/40 border-t pt-4">
              <ActionButton
                action={() => {
                  if (activeVersion) {
                    onStartInstall(activeVersion, instanceName);
                  }
                }}
                className="h-auto w-full gap-2 rounded-xl py-3 font-semibold text-sm"
                disabled={
                  !activeVersion || (data && data.length === 0) || !!error
                }
              >
                <HugeiconsIcon icon={Download01Icon} size={18} />
                {t("stepConfigureInstance.install")}
              </ActionButton>
            </div>
          </>
        )}
      </LoadingSwap>
    </div>
  );
}
