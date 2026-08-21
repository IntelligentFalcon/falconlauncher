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
import { useTranslation } from "react-i18next"; // <-- Import added
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
    tKey: string; // <-- Changed from title to tKey
    url: string;
    icon?: React.ReactNode;
    items?: {
        tKey: string; // <-- Changed from title to tKey
        url: string;
    }[];
}[] = [
    {
        icon: <HugeiconsIcon className="pointer-events-none shrink-0" icon={GameboyIcon} strokeWidth={2}/>,
        tKey: "main",
        url: "/",
    },
    {
        icon: <HugeiconsIcon className="pointer-events-none shrink-0" icon={Download01Icon} strokeWidth={2}/>,
        tKey: "download",
        url: "/downloads",
    },
    {
        icon: <HugeiconsIcon className="pointer-events-none shrink-0" icon={Package01Icon} strokeWidth={2}/>,
        tKey: "mods",
        url: "/mods",
    },
    {
        icon: <HugeiconsIcon className="pointer-events-none shrink-0" icon={Settings01Icon} strokeWidth={2}/>,
        tKey: "settings",
        url: "/settings",
    },
    {
        icon: <HugeiconsIcon className="pointer-events-none shrink-0" icon={ConsoleIcon} strokeWidth={2}/>,
        tKey: "console",
        url: "/console",
    },
];

export function NavMenu() {
    const { t } = useTranslation(); // <-- Initialize translation hook

    return (
        <SidebarGroup className="select-none">
            <SidebarGroupLabel>{t("navMenu.platform")}</SidebarGroupLabel> {/* <-- Translated */}
            <SidebarMenu>
                {NAVIGATION_ITEMS.map((item) => {
                    // Translate the main item title
                    const translatedTitle = t(`navMenu.items.${item.tKey}`);

                    return (
                        <NavLink key={item.tKey} to={item.url} draggable={false}>
                            {({isActive}) =>
                                item.items ? (
                                    <Collapsible
                                        className="group/collapsible"
                                        defaultOpen={isActive}
                                        render={<SidebarMenuItem/>}
                                    >
                                        <CollapsibleTrigger
                                            render={
                                                <SidebarMenuButton
                                                    isActive={isActive}
                                                    tooltip={translatedTitle}
                                                />
                                            }
                                        >
                                            {item.icon}
                                            <span>{translatedTitle}</span>
                                            <HugeiconsIcon
                                                className="pointer-events-none ms-auto shrink-0 transition-transform duration-200 group-data-open/collapsible:rotate-90"
                                                icon={ArrowRight01Icon}
                                                strokeWidth={2}
                                            />
                                        </CollapsibleTrigger>
                                        <CollapsibleContent>
                                            <SidebarMenuSub>
                                                {item.items?.map((subItem) => {
                                                    // Translate the sub-item title if it exists
                                                    const translatedSubTitle = t(`navMenu.items.${subItem.tKey}`);
                                                    return (
                                                        <SidebarMenuSubItem key={subItem.tKey}>
                                                            <SidebarMenuSubButton
                                                                render={<a href={subItem.url} draggable={false}/>}
                                                            >
                                                                <span>{translatedSubTitle}</span>
                                                            </SidebarMenuSubButton>
                                                        </SidebarMenuSubItem>
                                                    )})}
                                            </SidebarMenuSub>
                                        </CollapsibleContent>
                                    </Collapsible>
                                ) : (
                                    <SidebarMenuItem>
                                        <SidebarMenuButton isActive={isActive} tooltip={translatedTitle}>
                                            {item.icon}
                                            <span>{translatedTitle}</span>
                                        </SidebarMenuButton>
                                    </SidebarMenuItem>
                                )
                            }
                        </NavLink>
                    )})}
            </SidebarMenu>
        </SidebarGroup>
    );
}