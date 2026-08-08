import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/components/ui/sidebar';
import { HugeiconsIcon } from '@hugeicons/react';
import {
  UnfoldMoreIcon,
  PlusSignIcon,
  MicrosoftIcon,
  UserIcon,
  Delete01Icon,
} from '@hugeicons/core-free-icons';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useBackend, useBackendMutation } from '@/hooks/use-backend';
import { Field, FieldGroup } from './ui/field';
import { Label } from './ui/label';
import { Input } from './ui/input';
import { Button } from './ui/button';
import { Profile } from '@/invokes';
import { useEffect, useState } from 'react';

export function NavProfile() {
  const { isMobile } = useSidebar();

  const [profile, setProfile] = useState<Profile | null>(null);
  const [openCreateDialog, setOpenCreateDialog] = useState(false);
  const [newUsername, setNewUsername] = useState('');

  const { data: profiles, refetch: refetchProfiles } = useBackend({
    name: 'get_profiles',
    queryKey: ['profiles'],
  });

  const { data: selectedProfile } = useBackend({
    name: 'get_selected_profile',
    queryKey: ['selected_profile'],
  });

  useEffect(() => {
    if (!profile) {
      if (selectedProfile && selectedProfile.uuid) {
        const matchedProfile = profiles?.find((p) => p.uuid === selectedProfile.uuid);
        setProfile(matchedProfile || selectedProfile);
      } else if (profiles && profiles.length > 0) {
        setProfile(profiles[0]);
      }
    }
  }, [profiles, selectedProfile, profile]);

  // ADDED: Mutation to save the configuration/profile state
  // Replace 'save_config' with the exact name of your backend save command
  const { mutate: saveConfigMutation } = useBackendMutation({
    name: 'save',
  });

  // UPDATED: Added onSuccess to trigger the save command after the profile is set
  const { mutate: setProfileMutation } = useBackendMutation({
    name: 'set_selected_profile',
    onSuccess: () => {
      saveConfigMutation();
    },
  });

  useEffect(() => {
    if (profile) {
      setProfileMutation({ profile: profile });
    }
  }, [profile, setProfileMutation]);

  const { mutate: createOfflineProfile, isPending: isCreating } = useBackendMutation({
    name: 'create_offline_profile',
    onSuccess: () => {
      refetchProfiles();
      setNewUsername('');
      setOpenCreateDialog(false);
    },
  });

  const { mutate: removeProfileMutation } = useBackendMutation({
    name: 'remove_profile',
    onSuccess: () => {
      refetchProfiles();
    },
  });

  const handleCreateProfile = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = newUsername.trim();
    if (!trimmed) return;

    createOfflineProfile({ username: trimmed });
  };

  const handleRemoveProfile = (p: Profile, e: React.MouseEvent) => {
    e.stopPropagation();

    removeProfileMutation({ profile: p });

    if (profile?.uuid === p.uuid) {
      setProfile(null);
    }
  };

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
                  <HugeiconsIcon
                      icon={profile?.online ? MicrosoftIcon : UserIcon}
                  />
                </div>
                <div className="grid flex-1 text-start text-sm leading-tight">
                  <span className="truncate font-medium">{profile?.username}</span>
                  <span className="truncate text-xs">{profile?.uuid}</span>
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
                  {profiles?.map((p) => (
                      <DropdownMenuItem
                          key={p.uuid}
                          onClick={() => setProfile(p)}
                          className="gap-2 p-2 flex items-center justify-between group cursor-pointer"
                      >
                        <div className="flex items-center gap-2">
                          <div className="flex size-6 items-center justify-center rounded-md border">
                            <HugeiconsIcon
                                icon={p?.online ? MicrosoftIcon : UserIcon}
                            />
                          </div>
                          <span>{p.username}</span>
                        </div>

                        <div
                            role="button"
                            className="hidden group-hover:flex items-center justify-center p-1 rounded hover:bg-destructive/10 hover:text-destructive transition-colors"
                            onClick={(e) => handleRemoveProfile(p, e)}
                            title="Remove profile"
                        >
                          <HugeiconsIcon icon={Delete01Icon} className="size-4" />
                        </div>
                      </DropdownMenuItem>
                  ))}
                </DropdownMenuGroup>
                <DropdownMenuSeparator />
                <DropdownMenuGroup>
                  <DropdownMenuItem
                      className="gap-2 p-2 cursor-pointer"
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
            <form onSubmit={handleCreateProfile} className="space-y-4">
              <DialogHeader>
                <DialogTitle>Create profile</DialogTitle>
                <DialogDescription>Make Offline Profile</DialogDescription>
              </DialogHeader>
              <FieldGroup>
                <Field>
                  <Label htmlFor="username">Username</Label>
                  <Input
                      id="username"
                      name="username"
                      value={newUsername}
                      onChange={(e) => setNewUsername(e.target.value)}
                      placeholder="Enter offline username"
                      autoFocus
                      required
                  />
                </Field>
              </FieldGroup>
              <DialogFooter>
                <DialogClose render={<Button variant="outline" type="button">Cancel</Button>} />
                <Button type="submit" disabled={isCreating || !newUsername.trim()}>
                  {isCreating ? 'Creating...' : 'Create Profile'}
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </>
  );
}