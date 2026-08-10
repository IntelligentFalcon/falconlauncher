"use client";

import {
  ArrowRight01Icon,
  ConsoleIcon,
  Download01Icon,
  GameboyIcon,
  Package01Icon,
  Settings01Icon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import { NavLink } from "react-router";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";

const NAVIGATION_ITEMS: {
  title: string;
  url: string;
  icon?: React.ReactNode;
  items?: {
    title: string;
    url: string;
  }[];
}[] = [
  {
    icon: <HugeiconsIcon icon={GameboyIcon} strokeWidth={2} />,
    title: "Main",
    url: "/",
  },
  {
    icon: <HugeiconsIcon icon={Download01Icon} strokeWidth={2} />,
    title: "Download",
    url: "/downloads",
  },
  {
    icon: <HugeiconsIcon icon={Package01Icon} strokeWidth={2} />,
    title: "Mods",
    url: "/mods",
  },
  {
    icon: <HugeiconsIcon icon={Settings01Icon} strokeWidth={2} />,
    title: "Settings",
    url: "/settings",
  },
  {
    icon: <HugeiconsIcon icon={ConsoleIcon} strokeWidth={2} />,
    title: "Console",
    url: "/console",
  },
];

export function NavMenu() {
  return (
    <SidebarGroup>
      <SidebarGroupLabel>Platform</SidebarGroupLabel>
      <SidebarMenu>
        {NAVIGATION_ITEMS.map((item) => (
          <NavLink key={item.title} to={item.url}>
            {({ isActive }) =>
              item.items ? (
                <Collapsible
                  className="group/collapsible"
                  defaultOpen={isActive}
                  render={<SidebarMenuItem />}
                >
                  <CollapsibleTrigger
                    render={
                      <SidebarMenuButton
                        isActive={isActive}
                        tooltip={item.title}
                      />
                    }
                  >
                    {item.icon}
                    <span>{item.title}</span>
                    <HugeiconsIcon
                      className="ms-auto transition-transform duration-200 group-data-open/collapsible:rotate-90"
                      icon={ArrowRight01Icon}
                      strokeWidth={2}
                    />
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <SidebarMenuSub>
                      {item.items?.map((subItem) => (
                        <SidebarMenuSubItem key={subItem.title}>
                          <SidebarMenuSubButton
                            render={<a href={subItem.url} />}
                          >
                            <span>{subItem.title}</span>
                          </SidebarMenuSubButton>
                        </SidebarMenuSubItem>
                      ))}
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </Collapsible>
              ) : (
                <SidebarMenuItem>
                  <SidebarMenuButton isActive={isActive} tooltip={item.title}>
                    {item.icon}
                    <span>{item.title}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              )
            }
          </NavLink>
        ))}
      </SidebarMenu>
    </SidebarGroup>
  );
}
