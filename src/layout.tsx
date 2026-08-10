import { BookXIcon, XslFreeIcons } from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { QueryClientProvider } from "@tanstack/react-query";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Outlet } from "react-router";
import { AppSidebar } from "./components/app-sidebar";
import { ThemeProvider } from "./components/theme-provider";
import { Button } from "./components/ui/button";
import { Menubar } from "./components/ui/menubar";
import { SidebarProvider } from "./components/ui/sidebar";
import { Toaster } from "./components/ui/sonner";
import { queryClient } from "./lib/query-client";

export default function Layout() {
  const window = getCurrentWindow();

  return (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <SidebarProvider className="bg-sidebar">
          <AppSidebar />
          <div className="antaliased h-screen w-full">
            <div className="flex items-center justify-between pr-1 pl-4">
              <Menubar
                className="flex-1 rounded-none border-0"
                data-tauri-drag-region
              >
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
                <Button
                  onClick={() => window.minimize()}
                  size="icon-xs"
                  variant="outline"
                >
                  <HugeiconsIcon icon={XslFreeIcons} />
                </Button>
                <Button
                  onClick={() => window.destroy()}
                  size="icon-xs"
                  variant="destructive"
                >
                  <HugeiconsIcon icon={BookXIcon} />
                </Button>
              </div>
            </div>
            <div className="h-[calc(100%-2rem)] overflow-y-auto rounded-tl-2xl bg-background p-4">
              <Outlet />
            </div>
          </div>
        </SidebarProvider>
        <Toaster />
      </QueryClientProvider>
    </ThemeProvider>
  );
}
