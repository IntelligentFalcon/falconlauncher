import * as React from 'react';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/components/ui/sidebar';
import { HugeiconsIcon, IconSvgElement } from '@hugeicons/react';
import {
  UnfoldMoreIcon,
  PlusSignIcon,
  ProfileIcon,
  Profile02Icon,
  User02Icon,
  MicrosoftIcon,
  PresentationOnlineIcon,
} from '@hugeicons/core-free-icons';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { useBackend, useBackendMutation } from '@/hooks/use-backend';
import { Field, FieldGroup } from './ui/field';
import { Label } from './ui/label';
import { Input } from './ui/input';
import { Button } from './ui/button';

interface Profile {
  id: string;
  name: string;
  type: 'offline' | 'mojang' | 'microsoft';
}

const TYPE_ICONS = {
  microsoft: MicrosoftIcon,
  mojang: PresentationOnlineIcon,
  offline: User02Icon,
} satisfies Record<Profile['type'], IconSvgElement>;

const PROFILES: Profile[] = [
  {
    name: 'Gnkalk',
    id: 'gnkalk',
    type: 'offline',
  },
  {
    id: 'gnkalk@outlook.com',
    name: 'Gnkalk',
    type: 'microsoft',
  },
];

export function ProfileSwitcher() {
  const { isMobile } = useSidebar();

  const [profile, setProfile] = React.useState(PROFILES[0]);
  const [openCreateDialog, setOpenCreateDialog] = React.useState(false);

  // const { data: profiles } = useBackend({
  //   name: 'get_profiles',
  //   queryKey: ['profiles'],
  // });

  // const { data: profile, refetch: updatePorfile } = useBackend({
  //   name: 'get_username',
  //   queryKey: ['profiles', 'me'],
  // });

  // const { mutate: setProfile } = useBackendMutation({
  //   name: 'set_username',
  //   onSuccess: () => {
  //     updatePorfile();
  //   },
  // });

  return (
    <>
      <SidebarMenu>
        <SidebarMenuItem>
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <SidebarMenuButton
                  size="lg"
                  className="data-open:bg-sidebar-accent data-open:text-sidebar-accent-foreground group-data-[state=collapsed]:rounded-full"
                  tooltip="Switch Profile"
                />
              }
            >
              <div className="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                <HugeiconsIcon icon={TYPE_ICONS[profile.type]} />
              </div>
              <div className="grid flex-1 text-start text-sm leading-tight">
                <span className="truncate font-medium">{profile.name}</span>
                <span className="truncate text-xs">{profile.id}</span>
              </div>
              <HugeiconsIcon
                icon={UnfoldMoreIcon}
                strokeWidth={2}
                className="ms-auto"
              />
            </DropdownMenuTrigger>
            <DropdownMenuContent
              className="min-w-56 rounded-lg"
              align="start"
              side={isMobile ? 'bottom' : 'right'}
              sideOffset={4}
            >
              <DropdownMenuGroup>
                <DropdownMenuLabel className="text-xs text-muted-foreground">
                  Profiles
                </DropdownMenuLabel>
                {PROFILES.map((profile, index) => (
                  <DropdownMenuItem
                    key={profile.id}
                    onClick={() => setProfile(profile)}
                    className="gap-2 p-2"
                  >
                    <div className="flex size-6 items-center justify-center rounded-md border">
                      <HugeiconsIcon icon={TYPE_ICONS[profile.type]} />
                    </div>
                    {profile.name}
                    <DropdownMenuShortcut>⌘{index + 1}</DropdownMenuShortcut>
                  </DropdownMenuItem>
                ))}
              </DropdownMenuGroup>
              <DropdownMenuSeparator />
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className="gap-2 p-2"
                  onClick={() => setOpenCreateDialog(true)}
                >
                  <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                    <HugeiconsIcon
                      icon={PlusSignIcon}
                      strokeWidth={2}
                      className="size-4"
                    />
                  </div>
                  <div className="font-medium text-muted-foreground">
                    Add Profile
                  </div>
                </DropdownMenuItem>
              </DropdownMenuGroup>
            </DropdownMenuContent>
          </DropdownMenu>
        </SidebarMenuItem>
      </SidebarMenu>
      <Dialog open={openCreateDialog} onOpenChange={setOpenCreateDialog}>
        <DialogContent className="sm:max-w-sm">
          <form
            onSubmit={(e) => {
              e.preventDefault();
            }}
            className="space-y-4"
          >
            <DialogHeader>
              <DialogTitle>Create profile</DialogTitle>
              <DialogDescription>Make Offline Profile</DialogDescription>
            </DialogHeader>
            <FieldGroup>
              <Field>
                <Label htmlFor="username">Username</Label>
                <Input id="username" name="username" />
              </Field>
            </FieldGroup>
            <DialogFooter>
              <DialogClose render={<Button variant="outline">Cancel</Button>} />
              <Button type="submit">Create Profile</Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  );
}
