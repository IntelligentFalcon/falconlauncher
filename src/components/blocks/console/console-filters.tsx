import {
  Alert01Icon,
  AlertCircleIcon,
  Delete02Icon,
  InformationCircleIcon,
  LayersIcon,
  Search01Icon,
  ViewIcon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { ActionButton } from "@/components/ui/action-button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useConsoleActions, useConsoleState } from "@/context/console-context";

export function ConsoleFilters() {
  const { channels, filterChannel, filterLevel, searchQuery } =
    useConsoleState();
  const { onClearLogs, setFilterChannel, setFilterLevel, setSearchQuery } =
    useConsoleActions();
  const logLevels = [
    { icon: ViewIcon, label: "All Logs", name: "all" },
    { icon: InformationCircleIcon, label: "Info", name: "info" },
    { icon: Alert01Icon, label: "Warnings", name: "warn" },
    { icon: AlertCircleIcon, label: "Errors", name: "error" },
  ];

  return (
    <div className="flex shrink-0 flex-col gap-2 pb-2 sm:pb-3">
      <div className="flex w-full items-center justify-between gap-2 rounded-2xl border border-border/40 bg-secondary/30 p-1.5">
        <div className="flex min-w-0 flex-1 items-center gap-2">
          <div className="hidden shrink-0 items-center gap-1.5 px-2 font-bold text-[10px] text-muted-foreground uppercase tracking-wider sm:flex">
            <HugeiconsIcon icon={LayersIcon} size={12} strokeWidth={2.5} />
            Channels
          </div>

          <div className="scrollbar-none min-w-0 flex-1 overflow-x-auto">
            <Tabs
              onValueChange={(val) => setFilterChannel(val)}
              value={filterChannel}
            >
              <TabsList variant="line">
                {channels.map((chan) => (
                  <TabsTrigger
                    className="capitalize md:max-w-[200px]"
                    key={chan}
                    title={chan === "all" ? "All Channels" : chan}
                    value={chan}
                  >
                    <div
                      className={`h-2 w-2 shrink-0 rounded-full ${chan === "all" ? "bg-primary" : "bg-zinc-400"}`}
                    />
                    <span className="max-w-[120px] truncate">
                      {chan === "all" ? "ALL Channels" : chan}
                    </span>
                  </TabsTrigger>
                ))}
              </TabsList>
            </Tabs>
          </div>
        </div>

        <ActionButton
          action={async () => onClearLogs()}
          className="h-8 shrink-0 gap-1.5 rounded-xl px-2 text-xs sm:px-3"
          variant="destructive"
        >
          <HugeiconsIcon icon={Delete02Icon} size={14} strokeWidth={2} />
          <span className="hidden sm:inline">
            Clear {filterChannel === "all" ? "All" : filterChannel}
          </span>
        </ActionButton>
      </div>

      <div className="flex w-full flex-col items-stretch justify-between gap-2 rounded-2xl bg-secondary p-1.5 md:flex-row md:items-center">
        <div className="scrollbar-none w-full overflow-x-auto md:w-auto md:flex-1">
          <Tabs
            onValueChange={(val) => setFilterLevel(val)}
            value={filterLevel}
          >
            <TabsList className="p-0">
              {logLevels.map((lvl) => (
                <TabsTrigger
                  className="md:max-w-[120px]"
                  key={lvl.name}
                  title={lvl.label}
                  value={lvl.name}
                >
                  <HugeiconsIcon
                    className="shrink-0"
                    icon={lvl.icon}
                    strokeWidth={2}
                  />
                  <span className="inline">{lvl.label}</span>
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        </div>

        <div className="group relative flex h-8 w-full shrink-0 items-center rounded-xl border border-border/80 bg-background px-2.5 transition-colors focus-within:border-primary/60 md:w-64 md:max-w-xs">
          <HugeiconsIcon
            className="mr-1.5 shrink-0 text-muted-foreground transition-colors group-focus-within:text-primary"
            icon={Search01Icon}
            size={14}
            strokeWidth={2}
          />
          <input
            className="w-full min-w-0 border-none bg-transparent p-0 font-sans text-foreground text-xs outline-none placeholder:text-muted-foreground/60 focus:ring-0"
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search console logs..."
            type="text"
            value={searchQuery}
          />
          {searchQuery && (
            <button
              className="ml-1.5 shrink-0 rounded-md bg-secondary px-1.5 py-0.5 font-sans text-[10px] text-muted-foreground transition-colors hover:text-foreground"
              onClick={() => setSearchQuery("")}
              type="button"
            >
              Clear
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
