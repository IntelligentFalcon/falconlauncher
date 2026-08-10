import { ModsHeader } from "@/components/blocks/mods/mods-header";
import { ModsList as ModsListComponent } from "@/components/blocks/mods/mods-list";
import { ModsProvider } from "@/context/mods-context";

export default function Mods() {
  return (
    <ModsProvider>
      <div className="flex h-full flex-col space-y-4 p-2">
        <ModsHeader />

        <div className="flex flex-1 flex-col overflow-hidden overflow-y-auto rounded-2xl border border-border/40 bg-secondary/20 p-4">
          <ModsListComponent />
        </div>
      </div>
    </ModsProvider>
  );
}
