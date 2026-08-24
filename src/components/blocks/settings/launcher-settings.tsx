import {
  Settings01Icon,
  ToggleOffIcon,
  ToggleOnIcon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";
import {
  Combobox,
  ComboboxContent,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from "@/components/ui/combobox";
import { useLocale } from "@/stores/locale.ts";

const LANGUAGES = {
  en: "English (US)",
  fa: "فارسی",
} as const;

type LanguageKey = keyof typeof LANGUAGES;
const LANGUAGE_KEYS = Object.keys(LANGUAGES) as LanguageKey[];

export function LauncherSettings() {
  const { t } = useTranslation();
  const setLocale = useLocale((state) => state.setLocale);

  // Local state
  const [localLanguage, setLocalLanguage] = useState<LanguageKey | null>(null);
  const [localExitOnLaunch, setLocalExitOnLaunch] = useState<boolean | null>(null);
  const [localUseProxy, setLocalUseProxy] = useState<boolean | null>(null);
  const [localProxyAddress, setLocalProxyAddress] = useState<string>("");

  const exitOnLaunchQuery = useBackend({ name: "should_exit_on_launch" });
  const useProxyQuery = useBackend({ name: "should_use_proxy" });
  const getProxyQuery = useBackend({ name: "get_proxy" });

  const { mutateAsync: setExitMutation } = useBackendMutation({
    name: "set_exit_on_launch",
  });
  const { mutateAsync: setUseProxyMutation } = useBackendMutation({
    name: "set_use_proxy",
  });
  const { mutateAsync: setProxyMutation } = useBackendMutation({
    name: "set_proxy",
  });
  const { mutateAsync: saveMutation } = useBackendMutation({ name: "save" });

  const isQueriesLoading =
      exitOnLaunchQuery.isLoading ||
      useProxyQuery.isLoading ||
      getProxyQuery.isLoading;

  // Language resolution
  const backendLanguage = localLanguage as string | undefined;
  const rawLang = localLanguage ?? backendLanguage ?? "en";
  const language: LanguageKey = LANGUAGE_KEYS.includes(rawLang as LanguageKey)
      ? (rawLang as LanguageKey)
      : "en";

  // Exit on launch resolution
  const exitOnLaunch =
      localExitOnLaunch ?? (exitOnLaunchQuery.data as boolean) ?? false;

  // Proxy settings resolution
  const useProxy =
      localUseProxy ?? (useProxyQuery.data as boolean) ?? false;

  // Synchronize proxy server address when loaded
  useEffect(() => {
    if (getProxyQuery.data !== undefined && localProxyAddress === "") {
      setLocalProxyAddress(getProxyQuery.data as string);
    }
  }, [getProxyQuery.data]);

  const handleLanguageChange = async (lang: LanguageKey | null) => {
    if (!lang) return;
    setLocalLanguage(lang);
    setLocale(lang);
  };

  const handleExitToggle = async () => {
    const nextState = !exitOnLaunch;
    setLocalExitOnLaunch(nextState);
    await setExitMutation({ toggle: nextState });
    await saveMutation(undefined);
  };

  const handleProxyToggle = async () => {
    const nextState = !useProxy;
    setLocalUseProxy(nextState);
    await setUseProxyMutation({ toggle: nextState });
    await saveMutation(undefined);
  };

  const handleProxyAddressBlur = async () => {
    await setProxyMutation({ proxy: localProxyAddress });
    await saveMutation(undefined);
  };

  return (
      <LoadingSwap className="h-full w-full" isLoading={isQueriesLoading}>
        <div className="max-w-xl space-y-6">
          <div className="space-y-1">
            <h3 className="flex items-center gap-2 font-semibold text-foreground text-sm">
              <HugeiconsIcon
                  className="text-primary"
                  icon={Settings01Icon}
                  size={16}
              />{" "}
              {t("settings.title")}
            </h3>
            <p className="text-muted-foreground text-xs">
              {t("settings.description")}
            </p>
          </div>

          <div className="space-y-4">
            {/* Language Selection Section */}
            <div className="flex items-center justify-between rounded-xl border border-border/40 bg-secondary/20 p-4">
              <div>
                <div className="font-semibold text-xs">
                  {t("settings.interfaceLanguage")}
                </div>
                <div className="text-[11px] text-muted-foreground">
                  {t("settings.interfaceLanguageDesc")}
                </div>
              </div>

              <div className="w-40">
                <Combobox
                    autoHighlight
                    items={LANGUAGE_KEYS}
                    onValueChange={(newLang: LanguageKey | null) =>
                        handleLanguageChange(newLang)
                    }
                    value={language}
                >
                  <ComboboxInput
                      readOnly
                      value={language ? LANGUAGES[language] : "Select Language"}
                      className="h-10 w-full cursor-pointer select-none caret-transparent rounded-lg border border-[#333] bg-[#1a1a1a] px-3 font-medium text-white text-xs outline-none transition-all hover:border-gray-500 focus:border-primary focus:ring-1 focus:ring-primary"
                  />
                  <ComboboxContent className="border-[#333] bg-[#1a1a1a] text-white">
                    <ComboboxList>
                      {(langKey: LanguageKey) => (
                          <ComboboxItem
                              className="cursor-pointer hover:bg-[#333]"
                              key={langKey}
                              value={langKey}
                          >
                            <span>{LANGUAGES[langKey]}</span>
                          </ComboboxItem>
                      )}
                    </ComboboxList>
                  </ComboboxContent>
                </Combobox>
              </div>
            </div>

            {/* Exit On Launch Section */}
            <div className="flex items-center justify-between rounded-xl border border-border/40 bg-secondary/20 p-4">
              <div>
                <div className="font-semibold text-xs">
                  {t("settings.exitOnLaunch")}
                </div>
                <div className="text-[11px] text-muted-foreground">
                  {t("settings.exitOnLaunchDesc")}
                </div>
              </div>
              <button
                  className="relative border-none p-0 text-muted-foreground outline-none transition-colors hover:text-foreground"
                  onClick={handleExitToggle}
                  type="button"
              >
                <HugeiconsIcon
                    className={
                      exitOnLaunch ? "text-primary" : "text-muted-foreground/60"
                    }
                    icon={exitOnLaunch ? ToggleOnIcon : ToggleOffIcon}
                    size={32}
                />
              </button>
            </div>

            {/* Use Proxy Section */}
            <div className="space-y-3 rounded-xl border border-border/40 bg-secondary/20 p-4">
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-semibold text-xs">
                    {t("settings.useProxy")}
                  </div>
                  <div className="text-[11px] text-muted-foreground">
                    {t("settings.useProxyDesc")}
                  </div>
                </div>
                <button
                    className="relative border-none p-0 text-muted-foreground outline-none transition-colors hover:text-foreground"
                    onClick={handleProxyToggle}
                    type="button"
                >
                  <HugeiconsIcon
                      className={
                        useProxy ? "text-primary" : "text-muted-foreground/60"
                      }
                      icon={useProxy ? ToggleOnIcon : ToggleOffIcon}
                      size={32}
                  />
                </button>
              </div>

              {/* Proxy Input (Visible/Active when enabled) */}
              {useProxy && (
                  <div className="pt-2 border-t border-border/20">
                    <label className="block mb-1 text-[11px] font-medium text-muted-foreground">
                      {t("settings.proxyAddress")}
                    </label>
                    <input
                        type="text"
                        value={localProxyAddress}
                        placeholder="http://127.0.0.1:8080"
                        onChange={(e) => setLocalProxyAddress(e.target.value)}
                        onBlur={handleProxyAddressBlur}
                        className="h-9 w-full rounded-lg border border-[#333] bg-[#1a1a1a] px-3 text-xs text-white placeholder:text-muted-foreground/40 outline-none transition-all hover:border-gray-500 focus:border-primary focus:ring-1 focus:ring-primary"
                    />
                  </div>
              )}
            </div>
          </div>
        </div>
      </LoadingSwap>
  );
}