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
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useRef, useState } from "react";
import { ActionButton } from "@/components/ui/action-button";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

interface LogLine {
  channel: string;
  level: string;
  message: string;
  timestamp: string;
}

export default function Console() {
  const [filterLevel, setFilterLevel] = useState<string>("all");
  const [filterChannel, setFilterChannel] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [logs, setLogs] = useState<LogLine[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const logContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let active = true;
    let unlistenFn: (() => void) | null = null;

    invoke<LogLine[]>("get_log_history")
      .then((history) => {
        if (active) {
          setLogs(history);
          setIsLoading(false);
        }
      })
      .catch((err) => console.error("History pipeline failure:", err));

    const setupListener = async () => {
      const unlisten = await listen<LogLine>("launcher-log-stream", (event) => {
        if (active) {
          setLogs((prevLogs) => {
            const buffered = [...prevLogs, event.payload];
            return buffered.slice(-1000);
          });
        }
      });
      unlistenFn = unlisten;
    };

    setupListener();

    return () => {
      active = false;
      if (unlistenFn) {
        unlistenFn();
      }
    };
  }, []);

  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logs, filterLevel, filterChannel, searchQuery]);

  const channels = useMemo(() => {
    const uniqueChannels = new Set(logs.map((log) => log.channel));
    return ["all", ...Array.from(uniqueChannels)];
  }, [logs]);

  const filteredLogs = logs.filter((log) => {
    const matchesLevel =
      filterLevel === "all" || log.level.toLowerCase() === filterLevel;
    const matchesChannel =
      filterChannel === "all" || log.channel === filterChannel;
    const matchesSearch =
      log.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
      log.channel.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesLevel && matchesChannel && matchesSearch;
  });

  const logLevels = [
    { icon: ViewIcon, label: "All Logs", name: "all" },
    { icon: InformationCircleIcon, label: "Info", name: "info" },
    { icon: Alert01Icon, label: "Warnings", name: "warn" },
    { icon: AlertCircleIcon, label: "Errors", name: "error" },
  ];

  const getLogLevelStyles = (level: string) => {
    switch (level.toLowerCase()) {
      case "error":
        return "text-destructive font-semibold";
      case "warn":
        return "text-yellow-500 font-medium";
      case "debug":
        return "text-muted-foreground/70 italic";
      default:
        return "text-foreground";
    }
  };

  const handleClearLogs = async () => {
    try {
      if (filterChannel === "all") {
        await invoke("clear_log_history");
        setLogs([]);
      } else {
        await invoke("clear_log_history_channel", { channel: filterChannel });
        setLogs((prev) => prev.filter((log) => log.channel !== filterChannel));
      }
    } catch (err) {
      console.warn("Backend executed buffer modifications:", err);
      if (filterChannel === "all") {
        setLogs([]);
      } else {
        setLogs((prev) => prev.filter((log) => log.channel !== filterChannel));
      }
    }
  };

  return (
    // 1. overflow-hidden ensures the entire component NEVER scrolls
    <div className="flex h-full w-full min-w-0 flex-col overflow-hidden bg-background">
      {/* Top Bar Navigation & Controls - shrink-0 ensures this stays pinned and doesn't squish */}
      <div className="flex shrink-0 flex-col gap-2 pb-2 sm:pb-3">
        {/* Channel Filter Row & Actions */}
        <div className="flex w-full items-center justify-between gap-2 rounded-2xl border border-border/40 bg-secondary/30 p-1.5">
          <div className="flex min-w-0 flex-1 items-center gap-2">
            <div className="hidden shrink-0 items-center gap-1.5 px-2 font-bold text-[10px] text-muted-foreground uppercase tracking-wider sm:flex">
              <HugeiconsIcon icon={LayersIcon} size={12} strokeWidth={2.5} />
              Channels
            </div>

            <div className="scrollbar-none min-w-0 flex-1 overflow-x-auto">
              <Tabs
                onValueChange={(val) => setFilterChannel(val as string)}
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
            action={handleClearLogs}
            className="h-8 shrink-0 gap-1.5 rounded-xl px-2 text-xs sm:px-3"
            variant="destructive"
          >
            <HugeiconsIcon icon={Delete02Icon} size={14} strokeWidth={2} />
            <span className="hidden sm:inline">
              Clear {filterChannel === "all" ? "All" : filterChannel}
            </span>
          </ActionButton>
        </div>

        {/* Sub-Header Bar (Severity Filter & Search Bar) */}
        <div className="flex w-full flex-col items-stretch justify-between gap-2 rounded-2xl bg-secondary p-1.5 md:flex-row md:items-center">
          <div className="scrollbar-none w-full overflow-x-auto md:w-auto md:flex-1">
            <Tabs
              onValueChange={(val) => setFilterLevel(val as string)}
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
              >
                Clear
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Display Terminal Box Container - flex-1 gives it ALL remaining height, min-h-0 allows internal scrolling */}
      <div className="relative min-h-0 flex-1">
        <LoadingSwap className="absolute inset-0" isLoading={isLoading}>
          <div
            className="h-full w-full space-y-1 overflow-y-auto rounded-2xl border border-border bg-black p-2 font-mono text-[11px] text-zinc-200 selection:bg-zinc-700 sm:p-4 sm:text-xs"
            ref={logContainerRef}
          >
            {filteredLogs.length > 0 ? (
              filteredLogs.map((log, index) => (
                <div
                  className="flex flex-col whitespace-pre-wrap break-words rounded px-1 py-0.5 leading-relaxed transition-colors hover:bg-zinc-900/50 sm:flex-row sm:items-start"
                  key={index}
                >
                  <div className="mb-0.5 flex shrink-0 flex-wrap items-center sm:mb-0">
                    <span className="mr-1.5 shrink-0 select-none text-muted-foreground sm:mr-2">
                      [{log.timestamp}]
                    </span>
                    <span className="mr-1.5 shrink-0 select-none font-medium text-teal-500/90 sm:mr-2">
                      ({log.channel})
                    </span>
                    <span className="mr-1.5 shrink-0 select-none font-bold text-zinc-500 uppercase sm:mr-2">
                      [{log.level}]
                    </span>
                  </div>
                  <span className={`${getLogLevelStyles(log.level)} flex-1`}>
                    {log.message}
                  </span>
                </div>
              ))
            ) : (
              <div className="flex h-full items-center justify-center p-4 text-center text-muted-foreground italic">
                <div className="max-w-xs">
                  No logs match your current criteria.
                  <br />
                  <span className="mt-1 block break-words text-[10px] opacity-70 sm:text-[11px]">
                    (Level: "{filterLevel}" | Channel: "{filterChannel}"{" "}
                    {searchQuery && `| Query: "${searchQuery}"`})
                  </span>
                </div>
              </div>
            )}
          </div>
        </LoadingSwap>
      </div>
    </div>
  );
}
