import React from 'react';
import { ThemeProvider } from './components/theme-provider';
import { QueryClientProvider } from '@tanstack/react-query';
import { queryClient } from './lib/query-client';
import { SidebarProvider } from './components/ui/sidebar';
import { AppSidebar } from './components/app-sidebar';
import {
  Menubar,
  MenubarContent,
  MenubarGroup,
  MenubarItem,
  MenubarMenu,
  MenubarSeparator,
  MenubarShortcut,
  MenubarTrigger,
} from './components/ui/menubar';
import { Button } from './components/ui/button';
import { HugeiconsIcon } from '@hugeicons/react';
import { BookXIcon, XslFreeIcons } from '@hugeicons/core-free-icons';
import { Outlet } from 'react-router';

export default function Layout() {
  return (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <SidebarProvider>
          <AppSidebar />
          <div className="h-screen w-full antaliased ">
            <div
              className="pl-4 pr-1 tauri-drag-region bg-sidebar flex items-center justify-between "
              drag-region
            >
              <Menubar className="border-0  rounded-none ">
                {/* <MenubarMenu>
                  <MenubarTrigger>File</MenubarTrigger>
                  <MenubarContent>
                    <MenubarGroup>
                      <MenubarItem>
                        New Tab <MenubarShortcut>⌘T</MenubarShortcut>
                      </MenubarItem>
                      <MenubarItem>New Window</MenubarItem>
                    </MenubarGroup>
                    <MenubarSeparator />
                    <MenubarGroup>
                      <MenubarItem>Share</MenubarItem>
                      <MenubarItem>Print</MenubarItem>
                    </MenubarGroup>
                  </MenubarContent>
                </MenubarMenu> */}
              </Menubar>
              <div className="space-x-1">
                <Button variant="outline" size="icon-xs">
                  <HugeiconsIcon icon={XslFreeIcons} />
                </Button>
                <Button variant="destructive" size="icon-xs">
                  <HugeiconsIcon icon={BookXIcon} />
                </Button>
              </div>
            </div>
            <div className="h-screen overflow-y-auto">
              <Outlet />
            </div>
          </div>
        </SidebarProvider>
      </QueryClientProvider>
    </ThemeProvider>
  );
}
