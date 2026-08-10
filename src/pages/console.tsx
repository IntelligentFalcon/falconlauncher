import { ConsoleFilters } from "@/components/blocks/console/console-filters";
import { ConsoleViewer } from "@/components/blocks/console/console-viewer";
import { ConsoleProvider } from "@/context/console-context";

export default function Console() {
  return (
    <ConsoleProvider>
      <div className="flex h-full w-full min-w-0 flex-col overflow-hidden bg-background">
        <ConsoleFilters />
        <ConsoleViewer />
      </div>
    </ConsoleProvider>
  );
}
