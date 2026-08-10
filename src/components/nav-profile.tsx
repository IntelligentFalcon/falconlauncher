import {
  Delete01Icon,
  MicrosoftIcon,
  PlusSignIcon,
  UnfoldMoreIcon,
  UserIcon,
} from "@hugeicons/core-free-icons";
import { HugeiconsIcon } from "@hugeicons/react";
import type React from "react";
import { useEffect, useState } from "react";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from "@/components/ui/sidebar";
import { useBackend, useBackendMutation } from "@/hooks/use-backend";
import type { Profile } from "@/invokes";
import { Button } from "./ui/button";
import { Field, FieldGroup } from "./ui/field";
import { Input } from "./ui/input";
import { Label } from "./ui/label";

export function NavProfile() {
  const { isMobile } = useSidebar();

  const [profile, setProfile] = useState<Profile | null>(null);
  const [openCreateDialog, setOpenCreateDialog] = useState(false);
  const [newUsername, setNewUsername] = useState("");

  const { data: profiles, refetch: refetchProfiles } = useBackend({
    name: "get_profiles",
    queryKey: ["profiles"],
  });

  const { data: selectedProfile } = useBackend({
    name: "get_selected_profile",
    queryKey: ["selected_profile"],
  });

  useEffect(() => {
    if (!profile) {
      if (selectedProfile && selectedProfile.uuid) {
        const matchedProfile = profiles?.find(
          (p) => p.uuid === selectedProfile.uuid
        );
        setProfile(matchedProfile || selectedProfile);
      } else if (profiles && profiles.length > 0) {
        setProfile(profiles[0]);
      }
    }
  }, [profiles, selectedProfile, profile]);

  const { mutateAsync: saveConfigMutation } = useBackendMutation({
    name: "save",
  });

  const { mutate: setProfileMutation } = useBackendMutation({
    name: "set_selected_profile",
    onSuccess: () => {
      saveConfigMutation();
    },
  });

  useEffect(() => {
    if (profile) {
      setProfileMutation({ profile });
    }
  }, [profile, setProfileMutation]);

  const { mutate: createOfflineProfile, isPending: isCreating } =
    useBackendMutation({
      name: "create_offline_profile",
      onSuccess: () => {
        refetchProfiles();
        setNewUsername("");
        setOpenCreateDialog(false);
      },
    });

  const { mutate: removeProfileMutation } = useBackendMutation({
    name: "remove_profile",
    onSuccess: () => {
      refetchProfiles();
    },
  });

  const handleCreateProfile = (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = newUsername.trim();
    if (!trimmed) {
      return;
    }

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
                  className="data-open:bg-sidebar-accent data-open:text-sidebar-accent-foreground group-data-[state=collapsed]:rounded-full"
                  size="lg"
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
                <span className="truncate font-medium">
                  {profile?.username}
                </span>
                <span className="truncate text-xs">{profile?.uuid}</span>
              </div>
              <HugeiconsIcon
                className="ms-auto"
                icon={UnfoldMoreIcon}
                strokeWidth={2}
              />
            </DropdownMenuTrigger>
            <DropdownMenuContent
              align="start"
              className="min-w-56 rounded-lg"
              side={isMobile ? "bottom" : "right"}
              sideOffset={4}
            >
              <DropdownMenuGroup>
                <DropdownMenuLabel className="text-muted-foreground text-xs">
                  Profiles
                </DropdownMenuLabel>
                {profiles?.map((p) => (
                  <DropdownMenuItem
                    className="group flex cursor-pointer items-center justify-between gap-2 p-2"
                    key={p.uuid}
                    onClick={() => setProfile(p)}
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
                      className="hidden items-center justify-center rounded p-1 transition-colors hover:bg-destructive/10 hover:text-destructive group-hover:flex"
                      onClick={(e) => handleRemoveProfile(p, e)}
                      role="button"
                      title="Remove profile"
                    >
                      <HugeiconsIcon className="size-4" icon={Delete01Icon} />
                    </div>
                  </DropdownMenuItem>
                ))}
              </DropdownMenuGroup>
              <DropdownMenuSeparator />
              <DropdownMenuGroup>
                <DropdownMenuItem
                  className="cursor-pointer gap-2 p-2"
                  onClick={() => setOpenCreateDialog(true)}
                >
                  <div className="flex size-6 items-center justify-center rounded-md border bg-transparent">
                    <HugeiconsIcon
                      className="size-4"
                      icon={PlusSignIcon}
                      strokeWidth={2}
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

      <Dialog onOpenChange={setOpenCreateDialog} open={openCreateDialog}>
        <DialogContent className="sm:max-w-sm">
          <form className="space-y-4" onSubmit={handleCreateProfile}>
            <DialogHeader>
              <DialogTitle>Create profile</DialogTitle>
              <DialogDescription>Make Offline Profile</DialogDescription>
            </DialogHeader>
            <FieldGroup>
              <Field>
                <Label htmlFor="username">Username</Label>
                <Input
                  autoFocus
                  id="username"
                  name="username"
                  onChange={(e) => setNewUsername(e.target.value)}
                  placeholder="Enter offline username"
                  required
                  value={newUsername}
                />
              </Field>
            </FieldGroup>
            <DialogFooter>
              <DialogClose
                render={
                  <Button type="button" variant="outline">
                    Cancel
                  </Button>
                }
              />
              <Button
                disabled={isCreating || !newUsername.trim()}
                type="submit"
              >
                {isCreating ? "Creating..." : "Create Profile"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </>
  );
}
