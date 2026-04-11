import { useBackend } from '@/hooks/use-backend';
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

  const { data } = useBackend({
    name: 'load_categorized_versions',
  });

  console.log(data);

  return <div>downloads</div>;
}
