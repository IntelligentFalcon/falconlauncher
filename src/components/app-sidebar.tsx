import * as React from 'react';

import { NavMenu } from '@/components/nav-menu';
import { NavUser } from '@/components/nav-user';
import { ProfileSwitcher } from '@/components/profile-switcher';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarRail,
} from '@/components/ui/sidebar';

// This is sample data.
const data = {
  user: {
    name: 'shadcn',
    email: 'm@example.com',
    avatar: '/avatars/shadcn.jpg',
  },
};

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  return (
    <Sidebar collapsible="icon" {...props}>
      <SidebarHeader>
        <div className="flex gap-2 items-center justify-center group-data-[state=collapsed]:gap-0 mt-2">
          <img
            src="/icon.png"
            className="size-8 dark:brightness-100 dark:-hue-rotate-60"
          />
          <h2 className="font-bold text-2xl mt-1 group-data-[state=collapsed]:w-0 w-46 overflow-hidden transition-[width] line-clamp-1">
            Falcon Launcher
          </h2>
        </div>
      </SidebarHeader>
      <SidebarContent>
        <NavMenu />
      </SidebarContent>
      <SidebarFooter>
        <ProfileSwitcher />
        <NavUser user={data.user} />
      </SidebarFooter>
      <SidebarRail />
    </Sidebar>
  );
}
