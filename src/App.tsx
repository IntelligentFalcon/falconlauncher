import { useTranslation } from 'react-i18next';
import { useBackend } from './hooks/use-backend';
import { QueryClientProvider } from '@tanstack/react-query';
import { queryClient } from './lib/query-client';

export default function App() {
  const { t } = useTranslation();

  return (
    <QueryClientProvider client={queryClient}>
      <div className="min-h-screen antialiased">
        <VersionSelect />
      </div>
    </QueryClientProvider>
  );
}

function VersionSelect() {
  const { data: versions } = useBackend({
    name: 'get_versions',
  });

  const { data: installedVersions } = useBackend({
    name: 'get_installed_versions',
  });

  console.log(versions);
  console.log(installedVersions);

  return null;
}
