import { ModsHeader } from "@/components/blocks/mods/mods-header";
import { ModsList as ModsListComponent } from "@/components/blocks/mods/mods-list";
import { ModsProvider } from "@/context/mods-context";

export default function Mods() {
    return (
        <ModsProvider>
            {/* Added select-none here to cascade the no-highlight rule to the entire Mods page */}
            <div className="flex h-full flex-col gap-4 p-2 select-none">
                <ModsHeader />

                {/* REMOVED: flex-wrap, content-start, gap-4 */}
                {/* ADDED: flex-col (so the child can stretch vertically) */}
                <div className="flex flex-1 flex-col overflow-hidden overflow-y-auto rounded-2xl border border-border/40 bg-secondary/20 p-4">
                    <ModsListComponent />
                </div>
            </div>
        </ModsProvider>
    );
}