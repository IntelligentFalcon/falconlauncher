import { useBackend } from '@/hooks/use-backend';
import { invoke } from '@tauri-apps/api/core';
import { useState } from 'react';

export default function Downloads() {
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');
  const [filters, setFilters] = useState({
    forge: false,
    fabric: false,
    neo_forge: false,
    lite_loader: false,
  });
  const [activeMajorVersion, setActiveMajorVersion] = useState('1.21');

  const { data, isLoading } = useBackend({
    name: 'get_categorized_versions',
    args: {
      forge: false,
      fabric: false,
      liteLoader: false,
      neoForge: false
    }
  });

  invoke('debug', { text: `${data} ${isLoading}` });
  console.log(data, isLoading);

  return <div>downloads</div>;
}
