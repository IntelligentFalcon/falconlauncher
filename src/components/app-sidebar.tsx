import type * as React from "react";

import { NavMenu } from "@/components/nav-menu";
import { NavProfile } from "@/components/nav-profile";
import { NavUser } from "@/components/nav-user";
import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarHeader,
    SidebarRail,
} from "@/components/ui/sidebar";

// This is sample data.
const data = {
    user: {
        avatar: "/avatars/shadcn.jpg",
        email: "m@example.com",
        name: "shadcn",
    },
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
    return (
        // Added select-none to the entire sidebar to prevent text highlighting
        <Sidebar className="border-0! select-none" collapsible="icon" {...props}>
            <SidebarHeader>
                <div className="flex items-center justify-center gap-2 group-data-[state=collapsed]:gap-0">
                    <img
                        alt="Falcon Launcher"
                        // Added pointer-events-none and draggable={false} to completely disable image ghost dragging
                        className="pointer-events-none size-8 dark:brightness-100 dark:-hue-rotate-60"
                        draggable={false}
                        height={32}
                        src="/icon.png"
                        width={32}
                    />
                    <h2 className="mt-1 line-clamp-1 w-46 overflow-hidden font-bold text-2xl transition-[width] group-data-[state=collapsed]:w-0">
                        Falcon Launcher
                    </h2>
                </div>
            </SidebarHeader>
            <SidebarContent>
                <NavMenu />
            </SidebarContent>
            <SidebarFooter>
                <NavProfile />
                <NavUser user={data.user} />
            </SidebarFooter>
            <SidebarRail />
        </Sidebar>
    );
}