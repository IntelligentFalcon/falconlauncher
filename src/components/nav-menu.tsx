'use client';

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible';
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from '@/components/ui/sidebar';
import { HugeiconsIcon } from '@hugeicons/react';
import {
  ArrowRight01Icon,
  Download01Icon,
  GameboyIcon,
  ConsoleIcon, Settings01Icon,
} from '@hugeicons/core-free-icons';
import { NavLink } from 'react-router';

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
    title: 'Main',
    url: '/',
    icon: <HugeiconsIcon icon={GameboyIcon} strokeWidth={2} />,
  },
  {
    title: 'Download',
    url: '/downloads',
    icon: <HugeiconsIcon icon={Download01Icon} strokeWidth={2} />,
  },
  {
    title: 'Settings',
    url: '/settings',
    icon: <HugeiconsIcon icon={Settings01Icon} strokeWidth={2}/>
  },
  {
    title: 'Console',
    url: '/console',
    icon: <HugeiconsIcon icon={ConsoleIcon} strokeWidth={2}/>
  }
];

export function NavMenu() {
  return (
    <SidebarGroup>
      <SidebarGroupLabel>Platform</SidebarGroupLabel>
      <SidebarMenu>
        {NAVIGATION_ITEMS.map((item) => (
          <NavLink to={item.url} key={item.title}>
            {({ isActive }) =>
              item.items ? (
                <Collapsible
                  defaultOpen={isActive}
                  className="group/collapsible"
                  render={<SidebarMenuItem />}
                >
                  <CollapsibleTrigger
                    render={
                      <SidebarMenuButton
                        tooltip={item.title}
                        isActive={isActive}
                      />
                    }
                  >
                    {item.icon}
                    <span>{item.title}</span>
                    <HugeiconsIcon
                      icon={ArrowRight01Icon}
                      strokeWidth={2}
                      className="ms-auto transition-transform duration-200 group-data-open/collapsible:rotate-90"
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
                  <SidebarMenuButton tooltip={item.title} isActive={isActive}>
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
