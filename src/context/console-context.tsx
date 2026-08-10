import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";

export interface LogLine {
  channel: string;
  level: string;
  message: string;
  timestamp: string;
}

interface ConsoleState {
  channels: string[];
  filterChannel: string;
  filteredLogs: LogLine[];
  filterLevel: string;
  isLoading: boolean;
  searchQuery: string;
}

interface ConsoleActions {
  onClearLogs: () => void;
  setFilterChannel: (val: string) => void;
  setFilterLevel: (val: string) => void;
  setSearchQuery: (val: string) => void;
}

export const ConsoleStateContext = createContext<ConsoleState | null>(null);
export const ConsoleActionsContext = createContext<ConsoleActions | null>(null);

export function useConsoleState() {
  const context = useContext(ConsoleStateContext);
  if (!context) {
    throw new Error("useConsoleState must be used within ConsoleProvider");
  }
  return context;
}

export function useConsoleActions() {
  const context = useContext(ConsoleActionsContext);
  if (!context) {
    throw new Error("useConsoleActions must be used within ConsoleProvider");
  }
  return context;
}

export function ConsoleProvider({ children }: { children: React.ReactNode }) {
  const [filterLevel, setFilterLevel] = useState<string>("all");
  const [filterChannel, setFilterChannel] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [logs, setLogs] = useState<LogLine[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  useEffect(() => {
    const controller = new AbortController();

    invoke<LogLine[]>("get_log_history")
      .then((history) => {
        if (!controller.signal.aborted) {
          setLogs(history);
          setIsLoading(false);
        }
      })
      .catch((err) => console.error("History pipeline failure:", err));

    const unlistenPromise = listen<LogLine>("launcher-log-stream", (event) => {
      if (!controller.signal.aborted) {
        setLogs((prevLogs) => {
          const buffered = [...prevLogs, event.payload];
          return buffered.slice(-1000);
        });
      }
    });

    return () => {
      controller.abort();
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const channels = useMemo(() => {
    const uniqueChannels = new Set(logs.map((log) => log.channel));
    return ["all", ...Array.from(uniqueChannels)];
  }, [logs]);

  const filteredLogs = useMemo(
    () =>
      logs.filter((log) => {
        const matchesLevel =
          filterLevel === "all" || log.level.toLowerCase() === filterLevel;
        const matchesChannel =
          filterChannel === "all" || log.channel === filterChannel;
        const matchesSearch =
          log.message.toLowerCase().includes(searchQuery.toLowerCase()) ||
          log.channel.toLowerCase().includes(searchQuery.toLowerCase());
        return matchesLevel && matchesChannel && matchesSearch;
      }),
    [logs, filterLevel, filterChannel, searchQuery]
  );

  const handleClearLogs = useCallback(async () => {
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
  }, [filterChannel]);

  const actions = useMemo(
    () => ({
      onClearLogs: handleClearLogs,
      setFilterChannel,
      setFilterLevel,
      setSearchQuery,
    }),
    [handleClearLogs]
  );

  const state = useMemo(
    () => ({
      channels,
      filterChannel,
      filteredLogs,
      filterLevel,
      isLoading,
      searchQuery,
    }),
    [channels, filterChannel, filterLevel, filteredLogs, isLoading, searchQuery]
  );

  return (
    <ConsoleActionsContext.Provider value={actions}>
      <ConsoleStateContext.Provider value={state}>
        {children}
      </ConsoleStateContext.Provider>
    </ConsoleActionsContext.Provider>
  );
}
