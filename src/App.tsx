import { useTranslation } from 'react-i18next';
import { useBackend, useBackendMutation } from './hooks/use-backend';
import { QueryClientProvider, useQueryClient } from '@tanstack/react-query';
import { queryClient } from './lib/query-client';
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
} from '@/components/ui/combobox';
import {
  Item,
  ItemContent,
  ItemDescription,
  ItemTitle,
} from '@/components/ui/item';
import { Card, CardContent } from './components/ui/card';
import { ThemeProvider } from './components/theme-provider';
import { app } from '@tauri-apps/api';
import { useConfig } from './stores/config';
import { ActionButton } from './components/ui/action-button';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './components/ui/select';
import { useRef } from 'react';
import { Button } from '@/components/ui/button';
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
import { Field, FieldGroup } from '@/components/ui/field';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { HugeiconsIcon } from '@hugeicons/react';
import { UserPlus } from '@hugeicons/core-free-icons';

export default function App() {
  const { t } = useTranslation();

  return (
    <ThemeProvider>
      <QueryClientProvider client={queryClient}>
        <div className="min-h-screen antaliased flex items-center justify-center">
          <div className="max-w-sm space-y-4">
            <h1 className="text-4xl text-center mb-0">Falcon</h1>
            <h2 className="text-2xl text-center mb-8">Launcher</h2>
            <ProfileSelect />
            <VersionSelect />
            <PlayButton />
          </div>
        </div>
      </QueryClientProvider>
    </ThemeProvider>
  );
}

function ProfileSelect() {
  const queryClient = useQueryClient();

  const { data: profiles } = useBackend({
    name: 'get_profiles',
    queryKey: ['profiles'],
  });

  const { data: profile } = useBackend({
    name: 'get_username',
    queryKey: ['profiles', 'me'],
  });

  const { mutate } = useBackendMutation({
    name: 'set_username',
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['profiles', 'me'] });
    },
  });

  console.log(profile);

  return (
    <div className="flex items-center gap-1">
      <Select
        value={profile}
        onValueChange={(profile) => {
          if (profile) {
            mutate({
              username: profile,
            });
          }
        }}
      >
        <SelectTrigger className="w-full">
          <SelectValue placeholder="Select Profile" />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            {profiles?.map((profile) => (
              <SelectItem key={profile} value={profile}>
                {profile}
              </SelectItem>
            ))}
          </SelectGroup>
        </SelectContent>
      </Select>
      <CreateOfflineProfile />
    </div>
  );
}

export function CreateOfflineProfile() {
  const inputRef = useRef<HTMLInputElement>(null);
  const queryClient = useQueryClient();
  const { mutate: createOfflineProfile } = useBackendMutation({
    name: 'create_offline_profile',
    args: {
      username: inputRef.current?.value ?? '',
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['profiles'] });
    },
  });

  return (
    <Dialog>
      <DialogTrigger
        render={
          <Button variant="outline" size="icon">
            <HugeiconsIcon strokeWidth={2} icon={UserPlus} />
          </Button>
        }
      />
      <DialogContent className="sm:max-w-sm">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            console.log('Dda');
            createOfflineProfile();
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
              <Input id="username" name="username" ref={inputRef} />
            </Field>
          </FieldGroup>
          <DialogFooter>
            <DialogClose render={<Button variant="outline">Cancel</Button>} />
            <Button type="submit">Create Profile</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}

function VersionSelect() {
  const { version, setVersion } = useConfig();

  const { data: installedVersions } = useBackend({
    name: 'get_installed_versions',
    initialData: [],
    initialDataUpdatedAt: 0,
  });

  return (
    <Combobox
      items={installedVersions}
      autoHighlight
      value={version}
      onValueChange={(version) => setVersion(version)}
    >
      <ComboboxInput placeholder="Select a Version" className="w-full" />
      <ComboboxContent>
        <ComboboxEmpty>No items found.</ComboboxEmpty>
        <ComboboxList>
          {(version) => (
            <ComboboxItem key={version} value={version}>
              {version}
            </ComboboxItem>
          )}
        </ComboboxList>
      </ComboboxContent>
    </Combobox>
  );
}

function PlayButton() {
  const version = useConfig((state) => state.version);
  const profile = useConfig((state) => state.profile);

  const { mutateAsync } = useBackendMutation({
    name: 'play',
    args: {
      app,
      selectedVersion: version ?? '',
    },
  });

  return (
    <ActionButton
      action={async () => {
        await mutateAsync();
      }}
      disabled={version === null || profile === null}
      className="w-full"
    >
      Play
    </ActionButton>
  );
}
