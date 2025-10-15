import { invoke } from '@tauri-apps/api/core';
import { GridIcon, ListIcon } from 'lucide-react';
import { useState } from 'react';
import { Label } from '@/components/ui/label';
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { cn } from '@/lib/utils';
import { Button } from '../ui/button';

export function VersionSelectorPopup({
  close,
  onVersionSelect,
}: {
  close: () => void;
  onVersionSelect: (version: any) => void;
}) {
  const { t } = useTranslation();
  const [viewMode, setViewMode] = useState<'grid' | 'list'>('grid');

  const [filters, setFilters] = useState({
    forge: false,
    fabric: false,
    neoForge: false,
    liteLoader: false,
  });

  const [activeMajor, setActiveMajor] = useState('1.21');
  const [activeSpecific, setActiveSpecific] = useState<any>(null);

  const { data: versionsData, isLoading } = useQuery({
    queryKey: ['versions', filters],
    queryFn: () =>
      invoke<
        {
          name: string;
          versions: {
            id: string;
            date: string;
            base: string;
          }[];
        }[]
      >('load_categorized_versions', filters),
    select: (data) => {
      const v: Record<string, { v: string; d: string; base: string }[]> = {};
      for (const category of data) {
        v[category.name] = category.versions.map((x) => ({
          v: x.id,
          d: x.date,
          base: x.base,
        }));
      }
      return v;
    },
  });

  const majorVersions = Object.keys(versionsData || {});

  const handleInstall = () => {
    if (activeSpecific) {
      onVersionSelect(activeSpecific);
    }
    close();
  };

  return (
    <div className="flex h-full">
      <aside className="w-xs px-4 flex flex-col justify-between gap-2">
        <div className="space-y-3">
          <h2 className="text-xl font-bold mb-6">{t('mod_loaders')}</h2>
          <div className="flex items-center bg-zinc-700 p-3 rounded-md">
            <input
              type="checkbox"
              id="forge"
              className="w-5 h-5 accent-indigo-500 cursor-pointer"
              checked={filters.forge}
              onChange={() =>
                setFilters((prev) => ({ ...prev, forge: !prev.forge }))
              }
            />
            <label
              htmlFor="forge"
              className="mx-3 text-base cursor-pointer grow"
            >
              {t('install_forge')}
            </label>
          </div>
          <div className="flex items-center bg-zinc-700 p-3 rounded-md">
            <input
              type="checkbox"
              id="fabric"
              className="w-5 h-5 accent-indigo-500 cursor-pointer"
              checked={filters.fabric}
              onChange={() =>
                setFilters((prev) => ({ ...prev, fabric: !prev.fabric }))
              }
            />
            <label
              htmlFor="fabric"
              className="mx-3 text-base cursor-pointer grow"
            >
              {t('install_fabric')}
            </label>
          </div>
          <div className="flex items-center bg-zinc-700 p-3 rounded-md">
            <input
              type="checkbox"
              id="liteloader"
              className="w-5 h-5 accent-indigo-500 cursor-pointer"
              checked={filters.liteLoader}
              onChange={() =>
                setFilters((prev) => ({
                  ...prev,
                  liteLoader: !prev.liteLoader,
                }))
              }
            />
            <label
              htmlFor="liteloader"
              className="mx-3 text-base cursor-pointer grow"
            >
              {t('install_liteloader')}
            </label>
          </div>
          <div className="flex items-center bg-zinc-700 p-3 rounded-md">
            <input
              type="checkbox"
              id="neoforge"
              className="w-5 h-5 accent-indigo-500 cursor-pointer"
              checked={filters.neoForge}
              onChange={() =>
                setFilters((prev) => ({ ...prev, neoForge: !prev.neoForge }))
              }
            />
            <label
              htmlFor="neoforge"
              className="mx-3 text-base cursor-pointer grow"
            >
              {t('install_neoforge')}
            </label>
          </div>
        </div>
        <Button variant="success" onClick={handleInstall} className="w-full">
          {t('install_selected')}
        </Button>
      </aside>
      <main className="grow p-6 flex flex-col overflow-y-auto">
        <header className="flex justify-between items-center mb-6">
          <h1 className="text-3xl font-bold">
            {t('minecraft_version')} {activeMajor}
          </h1>
          <div className="flex space-x-2">
            <button
              onClick={() => setViewMode('grid')}
              className={`p-2 rounded-md transition-colors ${
                viewMode === 'grid'
                  ? 'bg-indigo-600 text-white'
                  : 'bg-zinc-700 text-zinc-400 hover:bg-zinc-600'
              }`}
            >
              <GridIcon size={20} />
            </button>
            <button
              onClick={() => setViewMode('list')}
              className={`p-2 rounded-md transition-colors ${
                viewMode === 'list'
                  ? 'bg-indigo-600 text-white'
                  : 'bg-zinc-700 text-zinc-400 hover:bg-zinc-600'
              }`}
            >
              <ListIcon size={20} />
            </button>
          </div>
        </header>
        {isLoading ? (
          <p>Loading versions...</p>
        ) : (
          versionsData &&
          (versionsData[activeMajor].length === 0 ? (
            <p>No versions found!</p>
          ) : (
            <RadioGroup
              defaultValue={versionsData[activeMajor][0].v}
              onValueChange={(value) => {
                setActiveSpecific(versionsData[activeMajor].find((x) => x.v === value));
              }}
              className={cn(
                viewMode === 'grid'
                  ? 'grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-2'
                  : 'flex flex-col'
              )}
            >
              {versionsData &&
                versionsData[activeMajor].map((item) =>
                  viewMode === 'grid' ? (
                    <GridVersionItem
                      key={item.v}
                      version={item.v}
                      date={item.d}
                    />
                  ) : (
                    <ListVersionItem
                      key={item.v}
                      version={item.v}
                      date={item.d}
                    />
                  )
                )}
            </RadioGroup>
          ))
        )}
      </main>
      <aside className="w-32 px-4 overflow-y-auto">
        <ul className="space-y-1">
          {majorVersions.map((v) => (
            <li
              key={v}
              onClick={() => setActiveMajor(v)}
              className={cn(
                'px-3 py-4 text-center font-bold text-lg rounded-md cursor-pointer transition-colors',
                activeMajor === v
                  ? 'bg-stone-800 text-white'
                  : 'text-zinc-400 hover:bg-zinc-700',
                v.length > 5 && 'text-sm'
              )}
            >
              {v}
            </li>
          ))}
        </ul>
      </aside>
    </div>
  );
}

function ListVersionItem({ version, date }: { version: string; date: string }) {
  return (
    <div className="border-stone-500 border-4 has-data-[state=checked]:border-purple-600 has-focus-visible:border-stone-400 has-focus-visible:ring-stone-400/50 relative w-full p-3 shadow-xs transition-[color,box-shadow] outline-none has-focus-visible:ring-[3px] cursor-pointer">
      <RadioGroupItem value={version} id={version} className="sr-only" />
      <Label
        htmlFor={version}
        className="text-stone-50 flex flex-col items-start after:absolute after:inset-0"
      >
        <h6>{version}</h6>
        <p className="text-stone-400 text-xs">{date}</p>
      </Label>
    </div>
  );
}

function GridVersionItem({ version, date }: { version: string; date: string }) {
  return (
    <div className="border-stone-500 border-4 has-data-[state=checked]:border-purple-600 has-focus-visible:border-stone-400 has-focus-visible:ring-stone-400/50 relative w-full p-3 shadow-xs transition-[color,box-shadow] outline-none has-focus-visible:ring-[3px] cursor-pointer">
      <RadioGroupItem value={version} id={version} className="sr-only" />
      <Label
        htmlFor={version}
        className="text-stone-50 flex flex-col items-start after:absolute after:inset-0"
      >
        <h6>{version}</h6>
        <p className="text-stone-400 text-xs">{date}</p>
      </Label>
    </div>
  );
}
