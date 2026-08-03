import { ActionButton } from '@/components/ui/action-button';
import { LoadingSwap } from '@/components/ui/animated/swapper';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { HugeiconsIcon } from '@hugeicons/react';
import {
  InformationCircleIcon,
  Alert01Icon,
  AlertCircleIcon,
  ViewIcon,
  LayersIcon,
  Delete02Icon,
  Search01Icon,
} from '@hugeicons/core-free-icons';
import { useEffect, useState, useRef, useMemo } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface LogLine {
  timestamp: string;
  level: string;
  message: string;
  channel: string;
}

export default function Console() {
  const [filterLevel, setFilterLevel] = useState<string>('all');
  const [filterChannel, setFilterChannel] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [logs, setLogs] = useState<LogLine[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const logContainerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let active = true;
    let unlistenFn: (() => void) | null = null;

    invoke<LogLine[]>('get_log_history')
      .then((history) => {
        if (active) {
          setLogs(history);
          setIsLoading(false);
        }
      })
      .catch((err) => console.error('History pipeline failure:', err));

    const setupListener = async () => {
      const unlisten = await listen<LogLine>('launcher-log-stream', (event) => {
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
      if (unlistenFn) unlistenFn();
    };
  }, []);

  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logs, filterLevel, filterChannel, searchQuery]);

  const channels = useMemo(() => {
    const uniqueChannels = new Set(logs.map((log) => log.channel));
    return ['all', ...Array.from(uniqueChannels)];
  }, [logs]);

  const filteredLogs = logs.filter((log) => {
    const matchesLevel =
      filterLevel === 'all' || log.level.toLowerCase() === filterLevel;
    const matchesChannel =
      filterChannel === 'all' || log.channel === filterChannel;
    const matchesSearch =
      log.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
      log.channel.toLowerCase().includes(searchQuery.toLowerCase());
    return matchesLevel && matchesChannel && matchesSearch;
  });

  const logLevels = [
    { name: 'all', label: 'All Logs', icon: ViewIcon },
    { name: 'info', label: 'Info', icon: InformationCircleIcon },
    { name: 'warn', label: 'Warnings', icon: Alert01Icon },
    { name: 'error', label: 'Errors', icon: AlertCircleIcon },
  ];

  const getLogLevelStyles = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error':
        return 'text-destructive font-semibold';
      case 'warn':
        return 'text-yellow-500 font-medium';
      case 'debug':
        return 'text-muted-foreground/70 italic';
      default:
        return 'text-foreground';
    }
  };

  const handleClearLogs = async () => {
    try {
      if (filterChannel === 'all') {
        await invoke('clear_log_history');
        setLogs([]);
      } else {
        await invoke('clear_log_history_channel', { channel: filterChannel });
        setLogs((prev) => prev.filter((log) => log.channel !== filterChannel));
      }
    } catch (err) {
      console.warn('Backend executed buffer modifications:', err);
      if (filterChannel === 'all') {
        setLogs([]);
      } else {
        setLogs((prev) => prev.filter((log) => log.channel !== filterChannel));
      }
    }
  };

  return (
    // 1. overflow-hidden ensures the entire component NEVER scrolls
    <div className="flex flex-col h-full w-full overflow-hidden min-w-0 bg-background">
      {/* Top Bar Navigation & Controls - shrink-0 ensures this stays pinned and doesn't squish */}
      <div className="flex flex-col gap-2 shrink-0 pb-2 sm:pb-3">
        {/* Channel Filter Row & Actions */}
        <div className="flex items-center justify-between bg-secondary/30 p-1.5 rounded-2xl border border-border/40 gap-2 w-full">
          <div className="flex items-center gap-2 min-w-0 flex-1">
            <div className="hidden sm:flex text-[10px] font-bold uppercase tracking-wider text-muted-foreground px-2 items-center gap-1.5 shrink-0">
              <HugeiconsIcon icon={LayersIcon} size={12} strokeWidth={2.5} />
              Channels
            </div>

            <div className="flex-1 overflow-x-auto scrollbar-none min-w-0">
              <Tabs
                value={filterChannel}
                onValueChange={(val) => setFilterChannel(val as string)}
              >
                <TabsList variant="line">
                  {channels.map((chan) => (
                    <TabsTrigger
                      key={chan}
                      value={chan}
                      title={chan === 'all' ? 'All Channels' : chan}
                      className="md:max-w-[200px] capitalize"
                    >
                      <div
                        className={`h-2 w-2 rounded-full shrink-0 ${chan === 'all' ? 'bg-primary' : 'bg-zinc-400'}`}
                      />
                      <span className="truncate max-w-[120px]">
                        {chan === 'all' ? 'ALL Channels' : chan}
                      </span>
                    </TabsTrigger>
                  ))}
                </TabsList>
              </Tabs>
            </div>
          </div>

          <ActionButton
            action={handleClearLogs}
            variant="destructive"
            className="h-8 text-xs gap-1.5 rounded-xl shrink-0 px-2 sm:px-3"
          >
            <HugeiconsIcon icon={Delete02Icon} size={14} strokeWidth={2} />
            <span className="hidden sm:inline">
              Clear {filterChannel === 'all' ? 'All' : filterChannel}
            </span>
          </ActionButton>
        </div>

        {/* Sub-Header Bar (Severity Filter & Search Bar) */}
        <div className="flex flex-col md:flex-row items-stretch md:items-center justify-between bg-secondary p-1.5 rounded-2xl w-full gap-2">
          <div className="overflow-x-auto scrollbar-none w-full md:w-auto md:flex-1">
            <Tabs
              value={filterLevel}
              onValueChange={(val) => setFilterLevel(val as string)}
            >
              <TabsList className="p-0">
                {logLevels.map((lvl) => (
                  <TabsTrigger
                    key={lvl.name}
                    value={lvl.name}
                    title={lvl.label}
                    className="md:max-w-[120px]"
                  >
                    <HugeiconsIcon
                      icon={lvl.icon}
                      strokeWidth={2}
                      className="shrink-0"
                    />
                    <span className="inline">{lvl.label}</span>
                  </TabsTrigger>
                ))}
              </TabsList>
            </Tabs>
          </div>

          <div className="relative flex items-center w-full md:max-w-xs md:w-64 h-8 bg-background border border-border/80 rounded-xl px-2.5 group focus-within:border-primary/60 transition-colors shrink-0">
            <HugeiconsIcon
              icon={Search01Icon}
              size={14}
              className="text-muted-foreground shrink-0 mr-1.5 group-focus-within:text-primary transition-colors"
              strokeWidth={2}
            />
            <input
              type="text"
              placeholder="Search console logs..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full bg-transparent text-xs font-sans text-foreground placeholder:text-muted-foreground/60 outline-none border-none p-0 focus:ring-0 min-w-0"
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery('')}
                className="text-[10px] bg-secondary text-muted-foreground hover:text-foreground px-1.5 py-0.5 rounded-md font-sans shrink-0 transition-colors ml-1.5"
              >
                Clear
              </button>
            )}
          </div>
        </div>
      </div>

      {/* Display Terminal Box Container - flex-1 gives it ALL remaining height, min-h-0 allows internal scrolling */}
      <div className="flex-1 min-h-0 relative">
        <LoadingSwap isLoading={isLoading} className="absolute inset-0">
          <div
            ref={logContainerRef}
            className="h-full w-full bg-black text-zinc-200 font-mono text-[11px] sm:text-xs p-2 sm:p-4 rounded-2xl overflow-y-auto border border-border selection:bg-zinc-700 space-y-1"
          >
            {filteredLogs.length > 0 ? (
              filteredLogs.map((log, index) => (
                <div
                  key={index}
                  className="whitespace-pre-wrap break-words leading-relaxed hover:bg-zinc-900/50 py-0.5 px-1 rounded transition-colors flex flex-col sm:flex-row sm:items-start"
                >
                  <div className="flex items-center flex-wrap shrink-0 sm:mb-0 mb-0.5">
                    <span className="text-muted-foreground mr-1.5 sm:mr-2 select-none shrink-0">
                      [{log.timestamp}]
                    </span>
                    <span className="text-teal-500/90 mr-1.5 sm:mr-2 select-none shrink-0 font-medium">
                      ({log.channel})
                    </span>
                    <span className="uppercase mr-1.5 sm:mr-2 select-none text-zinc-500 font-bold shrink-0">
                      [{log.level}]
                    </span>
                  </div>
                  <span className={`${getLogLevelStyles(log.level)} flex-1`}>
                    {log.message}
                  </span>
                </div>
              ))
            ) : (
              <div className="h-full flex items-center justify-center text-muted-foreground italic text-center p-4">
                <div className="max-w-xs">
                  No logs match your current criteria.
                  <br />
                  <span className="text-[10px] sm:text-[11px] opacity-70 break-words mt-1 block">
                    (Level: "{filterLevel}" | Channel: "{filterChannel}"{' '}
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
