import { useEffect, useRef, useState } from "react";
import { Check, Copy, Files, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next"; // <-- Import added
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { useConsoleActions, useConsoleState } from "@/context/console-context";

export function ConsoleViewer() {
  const { t } = useTranslation(); // <-- Initialize translation hook
  const { filterChannel, filterLevel, filteredLogs, isLoading, searchQuery } =
      useConsoleState();
  const { onClearLogs } = useConsoleActions();

  const logContainerRef = useRef<HTMLDivElement>(null);

  // Context Menu & Copy State
  const [contextMenu, setContextMenu] = useState({
    show: false,
    x: 0,
    y: 0,
    selectedText: "",
  });
  const [copiedType, setCopiedType] = useState<"selected" | "all" | null>(null);

  // Auto-scroll to bottom on mount
  useEffect(() => {
    const el = logContainerRef.current;
    el?.scrollTo({ top: el?.scrollHeight ?? 0 });
  }, []);

  // Close context menu when clicking anywhere else
  useEffect(() => {
    const closeMenu = () => setContextMenu((prev) => ({ ...prev, show: false }));
    window.addEventListener("click", closeMenu);
    return () => window.removeEventListener("click", closeMenu);
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

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    const selected = window.getSelection()?.toString().trim() || "";
    setContextMenu({
      show: true,
      x: e.clientX,
      y: e.clientY,
      selectedText: selected,
    });
  };

  const handleCopy = async (text: string, type: "selected" | "all") => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedType(type);
      setTimeout(() => setCopiedType(null), 2000);
    } catch (err) {
      console.error("Failed to copy text:", err);
    }
  };

  const handleCopyAll = () => {
    const allText = filteredLogs
        .map(
            (log) =>
                `[${log.timestamp}] (${log.channel}) [${log.level.toUpperCase()}] ${log.message}`
        )
        .join("\n");
    handleCopy(allText, "all");
  };

  return (
      <div className="relative min-h-0 flex-1">
        <LoadingSwap className="absolute inset-0" isLoading={isLoading}>
          <div
              className="h-full w-full space-y-1 overflow-y-auto rounded-2xl border border-border bg-black p-2 font-mono text-[11px] text-zinc-200 selection:bg-primary/50 selection:text-white sm:p-4 sm:text-xs"
              ref={logContainerRef}
              onContextMenu={handleContextMenu}
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
                      <span className={`${getLogLevelStyles(log.level)} min-w-0 flex-1`}>
                  {log.message}
                </span>
                    </div>
                ))
            ) : (
                <div className="flex h-full items-center justify-center p-4 text-center text-muted-foreground italic">
                  <div className="max-w-xs">
                    {t("consoleViewer.noLogsMatch")} {/* <-- Translated */}
                    <br />
                    <span className="mt-1 block break-words text-[10px] opacity-70 sm:text-[11px]">
                  ({t("consoleViewer.filterLevel")}: "{filterLevel}" | {t("consoleViewer.filterChannel")}: "{filterChannel}"{" "}
                      {searchQuery && `| ${t("consoleViewer.filterQuery")}: "${searchQuery}"`}) {/* <-- Translated labels */}
                </span>
                  </div>
                </div>
            )}
          </div>
        </LoadingSwap>

        {/* Custom Context Menu */}
        {contextMenu.show && (
            <div
                className="fixed z-50 flex min-w-[180px] flex-col gap-1 rounded-xl border border-border/50 bg-background/95 p-1.5 shadow-xl backdrop-blur-md animate-in fade-in zoom-in-95 duration-100"
                style={{
                  top: contextMenu.y,
                  left: contextMenu.x,
                  // Ensure the menu doesn't flow off the bottom/right of the screen
                  transform: `translate(calc(min(0px, 100vw - 100% - ${contextMenu.x}px - 10px)), calc(min(0px, 100vh - 100% - ${contextMenu.y}px - 10px)))`,
                }}
            >
              {contextMenu.selectedText && (
                  <button
                      onClick={() => handleCopy(contextMenu.selectedText, "selected")}
                      className="flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-xs font-medium text-foreground transition-colors hover:bg-secondary"
                  >
                    {copiedType === "selected" ? (
                        <Check className="h-3.5 w-3.5 text-emerald-500" />
                    ) : (
                        <Copy className="h-3.5 w-3.5 text-muted-foreground" />
                    )}
                    {copiedType === "selected" ? t("consoleViewer.copied") : t("consoleViewer.copySelected")} {/* <-- Translated */}
                  </button>
              )}

              <button
                  onClick={handleCopyAll}
                  className="flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-xs font-medium text-foreground transition-colors hover:bg-secondary"
              >
                {copiedType === "all" ? (
                    <Check className="h-3.5 w-3.5 text-emerald-500" />
                ) : (
                    <Files className="h-3.5 w-3.5 text-muted-foreground" />
                )}
                {copiedType === "all" ? t("consoleViewer.copiedAll") : t("consoleViewer.copyAllLogs")} {/* <-- Translated */}
              </button>

              <div className="mx-1 my-0.5 h-px bg-border/50" />

              <button
                  onClick={() => onClearLogs()}
                  className="flex w-full items-center gap-2 rounded-lg px-2.5 py-2 text-xs font-medium text-destructive transition-colors hover:bg-destructive/10"
              >
                <Trash2 className="h-3.5 w-3.5" />
                {t("consoleViewer.clearLogs")} {/* <-- Translated */}
              </button>
            </div>
        )}
      </div>
  );
}