import { useEffect, useRef } from "react";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { useConsoleState } from "@/context/console-context";

export function ConsoleViewer() {
  const { filterChannel, filterLevel, filteredLogs, isLoading, searchQuery } =
    useConsoleState();
  const logContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = logContainerRef.current;
    el?.scrollTo({ top: el?.scrollHeight ?? 0 });
  }, []);

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

  return (
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
                key={index.toString()}
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
  );
}
