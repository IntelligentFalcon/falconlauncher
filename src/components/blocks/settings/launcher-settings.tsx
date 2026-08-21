import {
  Settings01Icon,
  ToggleOffIcon,
  ToggleOnIcon
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { useState } from "react";
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

const LANGUAGES = {
  en: "English (US)",
  fa: "فارسی"
} as const;

type LanguageKey = keyof typeof LANGUAGES;
const LANGUAGE_KEYS = Object.keys(LANGUAGES) as LanguageKey[];

export function LauncherSettings() {
  const { t, i18n } = useTranslation();

  // Strictly typed state
  const [localLanguage, setLocalLanguage] = useState<LanguageKey | null>(null);
  const [localExitOnLaunch, setLocalExitOnLaunch] = useState<boolean | null>(null);

  const langQuery = useBackend({ name: "get_language" });
  const exitOnLaunchQuery = useBackend({ name: "should_exit_on_launch" });

  const { mutateAsync: setLangMutation } = useBackendMutation({
    name: "set_language",
  });
  const { mutateAsync: setExitMutation } = useBackendMutation({
    name: "set_exit_on_launch"
  });
  const { mutateAsync: saveMutation } = useBackendMutation({ name: "save" });

  const isQueriesLoading = langQuery.isLoading || exitOnLaunchQuery.isLoading;

  // Safely infer and validate the language key from the backend
  const backendLanguage = langQuery.data as string | undefined;
  const rawLang = localLanguage ?? backendLanguage ?? "en";
  const language: LanguageKey = LANGUAGE_KEYS.includes(rawLang as LanguageKey)
      ? (rawLang as LanguageKey)
      : "en";

  const exitOnLaunch = localExitOnLaunch ?? (exitOnLaunchQuery.data as boolean) ?? false;

  const handleLanguageChange = async (lang: LanguageKey | null) => {
    if (!lang) return;

    setLocalLanguage(lang);
    await i18n.changeLanguage(lang);
    await setLangMutation({ lang });
    await saveMutation(undefined);
  };

  const handleExitToggle = async () => {
    const nextState = !exitOnLaunch;
    setLocalExitOnLaunch(nextState);
    await setExitMutation({ toggle: nextState });
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
                    onValueChange={(newLang: LanguageKey | null) => handleLanguageChange(newLang)}
                    value={language}
                >
                  <ComboboxInput
                      readOnly
                      // 2. ADDED value here to show the correct text instead of just "en" or "fa"
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

            {/* Exit On Launch Section (Restored) */}
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
          </div>
        </div>
      </LoadingSwap>
  );
}