import {
  Download02Icon,
  GameController01Icon,
  Settings01Icon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { GameOptions } from "@/components/blocks/settings/game-options";
import { LauncherSettings } from "@/components/blocks/settings/launcher-settings";
import { MirrorSettings } from "@/components/blocks/settings/mirror-settings";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

export default function Settings() {
  return (
    <Tabs defaultValue="launcher">
      {/* Horizontal Navigation Header Layout */}
      <TabsList>
        {[
          { icon: Settings01Icon, id: "launcher", label: "Launcher Settings" },
          { icon: GameController01Icon, id: "game", label: "Game Options" },
          { icon: Download02Icon, id: "mirror", label: "Mirrors" },
        ].map((tab) => (
          <TabsTrigger key={tab.id} value={tab.id}>
            <HugeiconsIcon icon={tab.icon} size={16} strokeWidth={2} />
            <span className="font-medium text-xs">{tab.label}</span>
          </TabsTrigger>
        ))}
      </TabsList>

      {/* Config Panels Panel Box */}
      <div className="min-h-0 flex-1 overflow-y-auto rounded-2xl border border-border/60 bg-background/40 p-6">
        <TabsContent value="launcher">
          <LauncherSettings />
        </TabsContent>

        <TabsContent value="game">
          <GameOptions />
        </TabsContent>

        <TabsContent value="mirror">
          <MirrorSettings />
        </TabsContent>
      </div>
    </Tabs>
  );
}
