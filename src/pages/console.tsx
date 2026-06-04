import { ActionButton } from '@/components/ui/action-button';
import { LoadingSwap } from '@/components/ui/animated/swapper';
import { SidebarMenu, SidebarMenuItem, SidebarMenuButton } from '@/components/ui/sidebar';
import { HugeiconsIcon } from '@hugeicons/react';
import { InformationCircleIcon, Alert01Icon, AlertCircleIcon, ViewIcon, LayersIcon, Delete02Icon, Search01Icon } from '@hugeicons/core-free-icons';
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
    const [searchQuery, setSearchQuery] = useState<string>(''); // NEW: Search state tracker
    const [logs, setLogs] = useState<LogLine[]>([]);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const logContainerRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        let active = true;
        let unlistenFn: (() => void) | null = null;

        // 1. Fetch historical data cache from Rust state store
        invoke<LogLine[]>('get_log_history')
            .then((history) => {
                if (active) {
                    setLogs(history);
                    setIsLoading(false);
                }
            })
            .catch((err) => console.error("History pipeline failure:", err));

        // 2. Attach live stream listener simultaneously
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

    // Sticky scroll implementation
    useEffect(() => {
        if (logContainerRef.current) {
            logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
        }
    }, [logs, filterLevel, filterChannel, searchQuery]);

    // Extract unique channels dynamically from existing logs array
    const channels = useMemo(() => {
        const uniqueChannels = new Set(logs.map(log => log.channel));
        return ['all', ...Array.from(uniqueChannels)];
    }, [logs]);

    // Multi-axis filtering logic (Level + Channel + Search Text query matching)
    const filteredLogs = logs.filter((log) => {
        const matchesLevel = filterLevel === 'all' || log.level.toLowerCase() === filterLevel;
        const matchesChannel = filterChannel === 'all' || log.channel === filterChannel;
        const matchesSearch = log.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
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
            case 'error': return 'text-destructive font-semibold';
            case 'warn': return 'text-yellow-500 font-medium';
            case 'debug': return 'text-muted-foreground/70 italic';
            default: return 'text-foreground';
        }
    };

    const handleClearLogs = async () => {
        try {
            if (filterChannel === 'all') {
                await invoke('clear_log_history');
                setLogs([]);
            } else {
                await invoke('clear_log_history_channel', { channel: filterChannel });
                setLogs((prev) => prev.filter(log => log.channel !== filterChannel));
            }
        } catch (err) {
            console.warn("Backend executed buffer modifications:", err);
            if (filterChannel === 'all') {
                setLogs([]);
            } else {
                setLogs((prev) => prev.filter(log => log.channel !== filterChannel));
            }
        }
    };

    return (
        <div className="flex h-full w-full space-x-4">
            {/* Left Vertical Channel Sidebar Selector Lane */}
            <div className="w-48 bg-secondary/30 p-2 rounded-2xl flex flex-col justify-between shrink-0 border border-border/40">
                <div className="space-y-2">
                    <div className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground px-2 pt-1 flex items-center gap-1.5">
                        <HugeiconsIcon icon={LayersIcon} size={12} strokeWidth={2.5} />
                        Log Channels
                    </div>
                    <SidebarMenu className="space-y-1">
                        {channels.map((chan) => (
                            <SidebarMenuItem key={chan}>
                                <SidebarMenuButton
                                    onClick={() => setFilterChannel(chan)}
                                    isActive={filterChannel === chan}
                                    tooltip={chan === 'all' ? 'All Channels' : chan}
                                    className="w-full justify-start capitalize"
                                >
                                    <div className={`h-2 w-2 rounded-full ${chan === 'all' ? 'bg-primary' : 'bg-zinc-400'}`} />
                                    <span className="truncate">{chan === 'all' ? 'ALL Channels' : chan}</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        ))}
                    </SidebarMenu>
                </div>

                <ActionButton
                    action={handleClearLogs}
                    variant="destructive"
                    className="w-full h-9 text-xs gap-1.5 rounded-xl shrink-0 mt-4"
                >
                    <HugeiconsIcon icon={Delete02Icon} size={14} strokeWidth={2} />
                    Clear {filterChannel === 'all' ? 'All' : filterChannel}
                </ActionButton>
            </div>

            {/* Main Terminal panel */}
            <div className="flex-1 flex flex-col h-full space-y-3 min-w-0">
                {/* Top Header Filter Bar (Includes Severity + New Search input) */}
                <div className="flex flex-col sm:flex-row items-stretch sm:items-center justify-between bg-secondary p-1.5 rounded-2xl w-full gap-2">
                    <SidebarMenu className="flex-row items-center space-x-1 space-y-0 min-w-0 flex-1">
                        {logLevels.map((lvl) => (
                            <SidebarMenuItem key={lvl.name} className="flex-1 max-w-[120px]">
                                <SidebarMenuButton
                                    onClick={() => setFilterLevel(lvl.name)}
                                    isActive={filterLevel === lvl.name}
                                    tooltip={lvl.label}
                                    className="w-full justify-center md:justify-start"
                                >
                                    <HugeiconsIcon icon={lvl.icon} strokeWidth={2} />
                                    <span className="hidden sm:inline">{lvl.label}</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>
                        ))}
                    </SidebarMenu>

                    {/* NEW: Interactive Search Field Box */}
                    <div className="relative flex items-center max-w-xs w-full sm:w-64 h-8 bg-background border border-border/80 rounded-xl px-2.5 group focus-within:border-primary/60 transition-colors">
                        <HugeiconsIcon icon={Search01Icon} size={14} className="text-muted-foreground shrink-0 mr-1.5 group-focus-within:text-primary transition-colors" strokeWidth={2} />
                        <input
                            type="text"
                            placeholder="Search console logs..."
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            className="w-full bg-transparent text-xs font-sans text-foreground placeholder:text-muted-foreground/60 outline-none border-none p-0 focus:ring-0"
                        />
                        {searchQuery && (
                            <button
                                onClick={() => setSearchQuery('')}
                                className="text-[10px] bg-secondary text-muted-foreground hover:text-foreground px-1.5 py-0.5 rounded-md font-sans shrink-0 transition-colors"
                            >
                                Clear
                            </button>
                        )}
                    </div>
                </div>

                {/* Display Terminal Box Container */}
                <div className="flex-1 min-h-0">
                    <LoadingSwap isLoading={isLoading} className="h-full w-full">
                        <div
                            ref={logContainerRef}
                            className="h-full w-full bg-black text-zinc-200 font-mono text-xs p-4 rounded-2xl overflow-y-auto border border-border selection:bg-zinc-700 space-y-1"
                        >
                            {filteredLogs.length > 0 ? (
                                filteredLogs.map((log, index) => (
                                    <div key={index} className="whitespace-pre-wrap leading-relaxed hover:bg-zinc-900/50 py-0.5 px-1 rounded transition-colors flex items-start">
                                        <span className="text-muted-foreground mr-2 select-none shrink-0">[{log.timestamp}]</span>
                                        <span className="text-teal-500/90 mr-2 select-none shrink-0 font-medium">({log.channel})</span>
                                        <span className="uppercase mr-2 select-none text-zinc-500 font-bold shrink-0">[{log.level}]</span>
                                        <span className={getLogLevelStyles(log.level)}>{log.message}</span>
                                    </div>
                                ))
                            ) : (
                                <div className="h-full flex items-center justify-center text-muted-foreground italic text-center p-4">
                                    No logs match your current criteria.<br />
                                    <span className="text-[11px] opacity-70">
                                        (Level: "{filterLevel}" | Channel: "{filterChannel}" {searchQuery && `| Query: "${searchQuery}"`})
                                    </span>
                                </div>
                            )}
                        </div>
                    </LoadingSwap>
                </div>
            </div>
        </div>
    );
}