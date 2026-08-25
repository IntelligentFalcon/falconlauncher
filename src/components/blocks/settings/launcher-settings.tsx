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

type ProxyType = "none" | "http" | "socks5";

export function LauncherSettings() {
  const { t } = useTranslation();
  const setLocale = useLocale((state) => state.setLocale);

  // Local state
  const [localLanguage, setLocalLanguage] = useState<LanguageKey | null>(null);
  const [localExitOnLaunch, setLocalExitOnLaunch] = useState<boolean | null>(null);

  // Proxy state
  const [proxyType, setProxyType] = useState<ProxyType>("none");
  const [proxyHost, setProxyHost] = useState<string>("127.0.0.1");
  const [proxyPort, setProxyPort] = useState<string>("");
  const [useAuth, setUseAuth] = useState<boolean>(false);
  const [username, setUsername] = useState<string>("");
  const [password, setPassword] = useState<string>("");

  const exitOnLaunchQuery = useBackend({ name: "should_exit_on_launch" });
  const getProxyQuery = useBackend({ name: "get_proxy" });

  const { mutateAsync: setExitMutation } = useBackendMutation({
    name: "set_exit_on_launch",
  });
  const { mutateAsync: setProxyMutation, isPending: isSavingProxy } = useBackendMutation({
    name: "set_proxy",
  });
  const { mutateAsync: saveMutation } = useBackendMutation({ name: "save" });

  const isQueriesLoading = exitOnLaunchQuery.isLoading || getProxyQuery.isLoading;

  // Language resolution
  const backendLanguage = localLanguage as string | undefined;
  const rawLang = localLanguage ?? backendLanguage ?? "en";
  const language: LanguageKey = LANGUAGE_KEYS.includes(rawLang as LanguageKey)
      ? (rawLang as LanguageKey)
      : "en";

  // Exit on launch resolution
  const exitOnLaunch =
      localExitOnLaunch ?? (exitOnLaunchQuery.data as boolean) ?? false;

  // Populate proxy fields immediately when get_proxy data arrives
  useEffect(() => {
    if (getProxyQuery.data === undefined) return;

    const raw = ((getProxyQuery.data as string) || "").trim();

    if (!raw) {
      setProxyType("none");
      return;
    }

    try {
      // Ensure URL constructor can parse even if protocol wasn't prefixed
      const urlToParse = raw.includes("://") ? raw : `http://${raw}`;
      const parsed = new URL(urlToParse);

      const proto = raw.includes("://")
          ? parsed.protocol.replace(":", "").toLowerCase()
          : "http";

      if (proto.startsWith("socks5")) {
        setProxyType("socks5");
      } else {
        setProxyType("http");
      }

      setProxyHost(parsed.hostname || "127.0.0.1");
      setProxyPort(parsed.port || "");

      if (parsed.username || parsed.password) {
        setUseAuth(true);
        setUsername(decodeURIComponent(parsed.username));
        setPassword(decodeURIComponent(parsed.password));
      } else {
        setUseAuth(false);
        setUsername("");
        setPassword("");
      }
    } catch {
      // Fallback parser if standard URL parsing fails
      if (raw.toLowerCase().startsWith("socks5://")) {
        setProxyType("socks5");
      } else {
        setProxyType("http");
      }
      const stripped = raw.replace(/^(http|https|socks5|socks5h):\/\//i, "");
      const [hostPart, portPart] = stripped.split(":");
      setProxyHost(hostPart || "127.0.0.1");
      setProxyPort(portPart || "");
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

  const handleSaveProxy = async () => {
    let finalProxyString = "";

    if (proxyType !== "none") {
      const scheme = proxyType === "socks5" ? "socks5" : "http";
      const host = proxyHost.trim() || "127.0.0.1";
      const port = proxyPort.trim();
      const portSuffix = port ? `:${port}` : "";

      if (useAuth && (username || password)) {
        const encodedUser = encodeURIComponent(username);
        const encodedPass = encodeURIComponent(password);
        finalProxyString = `${scheme}://${encodedUser}:${encodedPass}@${host}${portSuffix}`;
      } else {
        finalProxyString = `${scheme}://${host}${portSuffix}`;
      }
    }

    await setProxyMutation({ proxy: finalProxyString });
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

            {/* Proxy Section */}
            <div className="space-y-4 rounded-xl border border-border/40 bg-secondary/20 p-4">
              <div>
                <div className="font-semibold text-xs">
                  {t("settings.proxySettings") || "Proxy Settings"}
                </div>
                <div className="text-[11px] text-muted-foreground">
                  {t("settings.proxySettingsDesc") ||
                      "Configure network proxy protocol and connection details"}
                </div>
              </div>

              {/* Protocol Radios */}
              <div className="flex items-center gap-6 pt-1">
                {(["none", "http", "socks5"] as const).map((type) => (
                    <label
                        key={type}
                        className="flex cursor-pointer items-center gap-2 text-xs font-medium text-foreground"
                    >
                      <input
                          type="radio"
                          name="proxyType"
                          value={type}
                          checked={proxyType === type}
                          onChange={() => setProxyType(type)}
                          className="accent-primary h-3.5 w-3.5 cursor-pointer"
                      />
                      {type === "none"
                          ? "None"
                          : type === "http"
                              ? "HTTP"
                              : "SOCKS5"}
                    </label>
                ))}
              </div>

              {/* Config Fields (Hidden when proxyType === 'none') */}
              {proxyType !== "none" && (
                  <div className="space-y-4 pt-2 border-t border-border/20">
                    {/* Host & Port Fields */}
                    <div className="grid grid-cols-3 gap-3">
                      <div className="col-span-2 space-y-1">
                        <label className="block text-[11px] font-medium text-muted-foreground">
                          {t("settings.proxyAddress") || "Address / Host"}
                        </label>
                        <input
                            type="text"
                            value={proxyHost}
                            placeholder="127.0.0.1"
                            onChange={(e) => setProxyHost(e.target.value)}
                            className="h-9 w-full rounded-lg border border-[#333] bg-[#1a1a1a] px-3 text-xs text-white placeholder:text-muted-foreground/40 outline-none transition-all hover:border-gray-500 focus:border-primary focus:ring-1 focus:ring-primary"
                        />
                      </div>
                      <div className="col-span-1 space-y-1">
                        <label className="block text-[11px] font-medium text-muted-foreground">
                          {t("settings.proxyPort") || "Port"}
                        </label>
                        <input
                            type="text"
                            value={proxyPort}
                            placeholder={proxyType === "socks5" ? "1080" : "8080"}
                            onChange={(e) => setProxyPort(e.target.value)}
                            className="h-9 w-full rounded-lg border border-[#333] bg-[#1a1a1a] px-3 text-xs text-white placeholder:text-muted-foreground/40 outline-none transition-all hover:border-gray-500 focus:border-primary focus:ring-1 focus:ring-primary"
                        />
                      </div>
                    </div>

                    {/* Authentication Toggle */}
                    <div className="space-y-3 pt-1">
                      <label className="flex cursor-pointer items-center gap-2 text-xs font-medium text-foreground">
                        <input
                            type="checkbox"
                            checked={useAuth}
                            onChange={(e) => setUseAuth(e.target.checked)}
                            className="accent-primary h-3.5 w-3.5 cursor-pointer rounded"
                        />
                        {t("settings.proxyAuth") || "Authentication"}
                      </label>

                      {useAuth && (
                          <div className="grid grid-cols-2 gap-3 pl-5 border-l-2 border-primary/30">
                            <div className="space-y-1">
                              <label className="block text-[11px] font-medium text-muted-foreground">
                                {t("settings.username") || "Username"}
                              </label>
                              <input
                                  type="text"
                                  value={username}
                                  placeholder="user"
                                  onChange={(e) => setUsername(e.target.value)}
                                  className="h-9 w-full rounded-lg border border-[#333] bg-[#1a1a1a] px-3 text-xs text-white placeholder:text-muted-foreground/40 outline-none transition-all hover:border-gray-500 focus:border-primary focus:ring-1 focus:ring-primary"
                              />
                            </div>
                            <div className="space-y-1">
                              <label className="block text-[11px] font-medium text-muted-foreground">
                                {t("settings.password") || "Password"}
                              </label>
                              <input
                                  type="password"
                                  value={password}
                                  placeholder="••••••••"
                                  onChange={(e) => setPassword(e.target.value)}
                                  className="h-9 w-full rounded-lg border border-[#333] bg-[#1a1a1a] px-3 text-xs text-white placeholder:text-muted-foreground/40 outline-none transition-all hover:border-gray-500 focus:border-primary focus:ring-1 focus:ring-primary"
                              />
                            </div>
                          </div>
                      )}
                    </div>
                  </div>
              )}

              {/* Save Proxy Button */}
              <div className="flex justify-end pt-2">
                <button
                    type="button"
                    onClick={handleSaveProxy}
                    disabled={isSavingProxy}
                    className="inline-flex h-8 items-center justify-center rounded-lg bg-primary px-4 text-xs font-medium text-primary-foreground transition-all hover:bg-primary/90 disabled:opacity-50"
                >
                  {isSavingProxy
                      ? t("settings.saving") || "Saving..."
                      : t("settings.saveProxy") || "Save Proxy"}
                </button>
              </div>
            </div>
          </div>
        </div>
      </LoadingSwap>
  );
}