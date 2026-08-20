import { Settings01Icon } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { useState } from "react";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";

export function LauncherSettings() {
  const [localLanguage, setLocalLanguage] = useState<string | null>(null);

  const langQuery = useBackend({ name: "get_language" });

  const { mutateAsync: setLangMutation } = useBackendMutation({
    name: "set_language",
  });
  const { mutateAsync: saveMutation } = useBackendMutation({ name: "save" });

  const isQueriesLoading = langQuery.isLoading;
  const language = localLanguage ?? (langQuery.data as string) ?? "en";

  const handleLanguageChange = async (lang: string) => {
    setLocalLanguage(lang);
    await setLangMutation({ lang });
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
              Launcher Preferences
            </h3>
            <p className="text-muted-foreground text-xs">
              Modify interface languages and general application settings.
            </p>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between rounded-xl border border-border/40 bg-secondary/20 p-4">
              <div>
                <div className="font-semibold text-xs">Interface Language</div>
                <div className="text-[11px] text-muted-foreground">
                  Swaps system core language string values.
                </div>
              </div>
              <select
                  className="cursor-pointer rounded-lg border border-border/80 bg-secondary px-3 py-1 font-medium text-foreground text-xs outline-none focus:border-primary"
                  onChange={(e) => handleLanguageChange(e.target.value)}
                  value={language}
              >
                <option value="en">English (US)</option>
                <option value="fr">Français</option>
                <option value="de">Deutsch</option>
                <option value="fa">فارسی</option>
              </select>
            </div>
          </div>
        </div>
      </LoadingSwap>
  );
}