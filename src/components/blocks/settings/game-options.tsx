import {
  GameController01Icon,
  ToggleOffIcon,
  ToggleOnIcon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { useState } from "react";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";

export function GameOptions() {
  const [localLanguage, setLocalLanguage] = useState<string | null>(null);
  const [localExitOnLaunch, setLocalExitOnLaunch] = useState<boolean | null>(
    null
  );

  const langQuery = useBackend({ name: "get_language" });
  const exitOnLaunchQuery = useBackend({ name: "should_exit_on_launch" });

  const { mutateAsync: setLangMutation } = useBackendMutation({
    name: "set_language",
  });
  const { mutateAsync: setExitMutation } = useBackendMutation({
    name: "set_exit_on_launch",
  });
  const { mutateAsync: saveMutation } = useBackendMutation({ name: "save" });

  const isQueriesLoading = langQuery.isLoading || exitOnLaunchQuery.isLoading;

  const language = localLanguage ?? (langQuery.data as string) ?? "en";
  const exitOnLaunch =
    localExitOnLaunch ?? (exitOnLaunchQuery.data as boolean) ?? false;

  const handleLanguageChange = async (lang: string) => {
    setLocalLanguage(lang);
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
              icon={GameController01Icon}
              size={16}
            />{" "}
            Runtime Preferences
          </h3>
          <p className="text-muted-foreground text-xs">
            Modify interface languages and automated window cloaking parameters.
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

          <div className="flex items-center justify-between rounded-xl border border-border/40 bg-secondary/20 p-4">
            <div>
              <div className="font-semibold text-xs">
                Exit Launcher on Launch
              </div>
              <div className="text-[11px] text-muted-foreground">
                Kills the application runtime once the subprocess completes
                assembly boot.
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
