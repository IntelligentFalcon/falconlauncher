import { useCallback, useEffect, useState } from 'react';
import {
  Grid,
  Home,
  List,
  Package,
  Play,
  Settings,
  X,
  Plus,
  Trash2,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import LoginPopup from './LoginPopup';
import { publicDir } from '@tauri-apps/api/path';
import { Button } from './components/ui/button';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from './components/ui/select';
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from './components/ui/dialog';
import { useTranslation } from 'react-i18next';
import { LocaleButton } from './components/basic/locale-button';
import { VersionSelectorPopup } from './components/block/version-manager';

export default function FalconLauncher() {
  const { t } = useTranslation();

  const [activeTab, setActiveTab] = useState('home');
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [isDownloading, setIsDownloading] = useState(false);
  const [versions, setVersions] = useState<string[]>([]);
  const [selectedVersion, setSelectedVersion] = useState('');
  const [username, setUsername] = useState('');
  const [statusMessage, setStatusMessage] = useState('Ready to play');
  const [isLoginPopupOpen, setIsLoginPopupPopupOpen] = useState(false);
  const [profiles, setProfiles] = useState([]);

  const [isVersionSelectorOpen, setIsVersionSelectorOpen] = useState(false);

  const loadVersions = useCallback(async () => {
    try {
      const v = await invoke('get_versions');
      setVersions(v);
      if (v.length > 0 && !selectedVersion) {
        setSelectedVersion(v[0]);
      }
    } catch (e) {
      console.error('Failed to fetch versions:', e);
    }
  }, [selectedVersion]);

  function reloadProfiles() {
    useEffect(() => {
      invoke('get_profiles')
        .then((v) => {
          setProfiles(v);
          if (v.length > 0) {
            invoke('get_username').then(setUsername).catch(console.error);
          }
        })
        .catch(console.error);
    }, [profiles, username]);
  }

  reloadProfiles();

  useEffect(() => {
    // // Mock translations for popup
    // const newTranslations = {
    //     en: { ...t.en, install_new_version: 'Install New Version', mod_loaders: 'Mod Loaders', install_selected: 'Install Selected', minecraft_version: 'Minecraft', install_forge: 'Install Forge', install_fabric: 'Install Fabric', install_liteloader: 'Install LiteLoader', install_neoforge: 'Install NeoForge' },
    //     fa: { ...t.fa, install_new_version: 'نصب نسخه جدید', mod_loaders: 'ماد لودرها', install_selected: 'نصب نسخه انتخابی', minecraft_version: 'ماینکرفت', install_forge: 'نصب Forge', install_fabric: 'نصب Fabric', install_liteloader: 'نصب LiteLoader', install_neoforge: 'نصب NeoForge' }
    // };
    // Object.assign(t, newTranslations);

    loadVersions()
      .then(() => console.log('loaded versions!'))
      .catch('Error!');

    let unlistenProgress, unlistenProgressBar;
    const registerEvents = async () => {
      unlistenProgress = await listen('progress', (event) =>
        setStatusMessage(event.payload)
      );
      unlistenProgressBar = await listen('progressBar', (event) => {
        if (event.payload >= 100) setIsDownloading(false);
        setDownloadProgress(event.payload);
      });
    };

    registerEvents().then(() => console.log('Done!'));

    return () => {
      unlistenProgress?.();
      unlistenProgressBar?.();
    };
  }, [loadVersions]);

  const handlePlay = async () => {
    if (!selectedVersion && versions.length > 0)
      setSelectedVersion(versions[0]);
    setIsDownloading(true);
    try {
      await invoke('set_username', { username: username });
      await invoke('save');
      await invoke('play_button_handler', { selectedVersion: selectedVersion });
    } catch (e) {
      console.error('Failed to launch game:', e);
      setIsDownloading(false);
    }
  };

  return (
    <div className="flex flex-col w-full h-screen bg-gray-900 text-gray-200 overflow-hidden">
      <div className="flex justify-between items-center px-4 sm:px-6 py-3 bg-gray-800 border-b border-gray-700">
        <div className="flex items-center flex-wrap gap-2">
          <h1 className="text-lg sm:text-xl font-bold text-indigo-400">
            {t('app_name')}
          </h1>
          <span className="text-xs text-gray-400">v1.0.0</span>
        </div>
        <div className="flex items-center">
          <LocaleButton />
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        <div className="w-full lg:w-64 md:w-48 bg-gray-800 flex flex-col">
          <div className="p-4 sm:p-6 flex flex-col">
            {/*WEIRD ISSUE HERE Background gray color is not working*/}
            <Select
              onValueChange={async (value) => {
                setUsername(value);
                await invoke('set_username', { username: value });
              }}
            >
              <SelectTrigger>
                <SelectValue
                  placeholder={username ?? profiles[0] ?? t('select_profile')}
                />
              </SelectTrigger>
              <SelectContent>
                {profiles.map((profile) => (
                  <SelectItem
                    key={profile}
                    value={profile}
                    className="capitalize"
                  >
                    {profile}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Button
              variant="secondary"
              className="text-xs"
              onClick={() => setIsLoginPopupPopupOpen(true)}
            >
              {t('create_profile')}
            </Button>

            <div className="border-t border-gray-700 pt-4">
              <h3 className="text-sm font-semibold mb-2 text-gray-400">
                {t('game_version')}
              </h3>
              <Select onValueChange={(value) => setSelectedVersion(value)}>
                <SelectTrigger>
                  <SelectValue
                    placeholder={selectedVersion ?? versions[0] ?? 'Loading...'}
                  />
                </SelectTrigger>
                <SelectContent>
                  {versions.map((version) => (
                    <SelectItem
                      key={version}
                      value={version}
                      className="capitalize"
                    >
                      {version}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Dialog>
                <DialogTrigger asChild>
                  <Button className="w-full" variant="secondary">
                    {t('install_new_version')}
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>{t('version_installer_title')}</DialogTitle>
                  </DialogHeader>
                  <DialogBody>
                    <VersionSelectorPopup
                      close={() => setIsVersionSelectorOpen(false)}
                      onVersionSelect={(version) =>
                        invoke('download_version', {
                          versionLoader: {
                            id: version.v,
                            date: version.d,
                            base: version.base,
                          },
                        })
                          .catch(() =>
                            console.error('Failed to download version')
                          )
                          .then(() => {
                            window.location.reload();
                          })
                      }
                    />
                  </DialogBody>
                </DialogContent>
              </Dialog>
            </div>
          </div>

          <nav className="flex-1 py-4 mt-auto">
            <NavItem
              icon={<Home size={18} />}
              title={t('home_tab')}
              active={activeTab === 'home'}
              onClick={() => setActiveTab('home')}
            />
            <NavItem
              icon={<Package size={18} />}
              title={t('mods_tab')}
              active={activeTab === 'mods'}
              onClick={() => setActiveTab('mods')}
            />
            <NavItem
              icon={<Settings size={18} />}
              title={t('settings_tab')}
              active={activeTab === 'settings'}
              onClick={() => setActiveTab('settings')}
            />
          </nav>

          <div className="p-4 sm:p-6 border-t border-gray-700">
            <Button
              disabled={isDownloading || username === ''}
              variant="success"
              className="w-full"
              onClick={handlePlay}
            >
              <Play size={18} className="mr-2" />
              {isDownloading ? t('downloading') : t('play')}
            </Button>
            {isDownloading && (
              <div className="w-full bg-gray-700 rounded-full h-2 mt-4">
                <div
                  className="bg-indigo-500 h-2 rounded-full"
                  style={{ width: `${downloadProgress}%` }}
                ></div>
              </div>
            )}
            <p className="text-xs mt-2 text-gray-400 text-center">
              {statusMessage}
            </p>
          </div>
        </div>

        <main className="flex-1 overflow-auto p-4 sm:p-6">
          {activeTab === 'home' && <HomeTab />}
          {activeTab === 'settings' && <SettingsTab />}
          {activeTab === 'mods' && <ModsTab />}
        </main>
      </div>

      <LoginPopup
        isOpen={isLoginPopupOpen}
        onClose={() => {
          setIsLoginPopupPopupOpen(false);
          reloadProfiles();
        }}
      />
    </div>
  );
}

// Other components (NavItem, HomeTab, ModsTab, SettingsTab, etc.) remain the same
function NavItem({ icon, title, active, onClick }) {
  return (
    <div
      className={`flex items-center px-6 py-3 cursor-pointer ${
        active
          ? 'bg-gray-700 border-r-4 border-indigo-500'
          : 'hover:bg-gray-700'
      }`}
      onClick={onClick}
    >
      <div className={`ml-3 ${active ? 'text-indigo-400' : 'text-gray-400'}`}>
        {icon}
      </div>
      <span className={`${active ? 'font-semibold' : ''}`}>{title}</span>
    </div>
  );
}

function HomeTab() {
  const { t } = useTranslation();

  const newsArticles = [
    {
      title: 'مهم',
      content:
        'در نسخه الفا ممکنه مشکلات زیادی ، کم و کسری زیادی باشه اگه چیزی به ذهنتون رسید و فیدبکی داشتید از گفتنش پرهیز نکنید @IntelligentFalcon',
      date: 'کمی پیش :)',
    },
  ];

  return (
    <div className="p-8">
      <h2 className="text-2xl font-bold mb-6">{t('minecraft_news')}</h2>

      <div className="space-y-4">
        {newsArticles.map((article, index) => (
          <div key={index} className="bg-gray-800 p-6 rounded-sm">
            <h3 className="text-xl font-semibold mb-2">{article.title}</h3>
            <p className="text-gray-300 mb-3">{article.content}</p>
            <p className="text-sm text-indigo-400 italic">{article.date}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

function AddModPopup({ isOpen, onClose }) {
  const { t } = useTranslation();

  if (!isOpen) {
    return null;
  }

  const handleInstallMod = () => {
    invoke('install_mod_from_local').catch('Failed to install mod from local');
    console.log('Install mod clicked');
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-70 flex justify-center items-center z-50">
      <div className="bg-gray-800 p-8 rounded-lg shadow-xl w-full max-w-sm relative text-gray-200">
        <button
          onClick={onClose}
          className="absolute top-4 right-4 text-gray-400 hover:text-white transition-colors"
        >
          <X size={24} />
        </button>
        <h2 className="text-3xl font-bold text-center mb-6">
          {t('install_mod')}
        </h2>
        <div className="space-y-4">
          <button
            onClick={handleInstallMod}
            className="w-full p-3 bg-green-600 hover:bg-green-700 text-white font-bold rounded-sm transition-colors"
          >
            {t('select_mod_file')}
          </button>
          <button
            onClick={() => console.log('Download from Modrinth clicked')}
            className="w-full p-3 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-sm transition-colors"
          >
            Download from Modrinth
          </button>
          <button
            onClick={() => console.log('Download from CurseForge clicked')}
            className="w-full p-3 bg-orange-600 hover:bg-orange-700 text-white font-bold rounded-sm transition-colors"
          >
            Download from CurseForge
          </button>
        </div>
      </div>
    </div>
  );
}

function ModsTab() {
  const { t } = useTranslation();

  const [mods, setMods] = useState([]);
  const [isAddModPopupOpen, setAddModPopupOpen] = useState(false);
  useEffect(() => {
    if (mods.length < 1)
      invoke('get_mods')
        .then((v) => {
          setMods(v);
        })
        .finally(() => {
          console.log('Loaded mods');
        });
  }, [mods]);

  const handleToggleMod = (mod, enabled) => {
    // Here you would invoke a Tauri command to enable/disable the mod
    mod.enabled = enabled;
    console.log(`Toggling mod ${mod.name} to ${enabled}`);
    invoke('toggle_mod', { modInfo: mod, toggle: enabled }).catch('Error!');
    // For now, we'll just update the local state for visual feedback
    setMods(mods.map((m) => (m.name === mod.name ? mod : m)));
  };

  const handleDeleteMod = (mod) => {
    console.log(`Deleting mod ${mod.name}`);
    // Here you would invoke a Tauri command to delete the mod
    // invoke("delete_mod", { modInfo: mod }).catch("Error!");
    // For now, we'll just update the local state for visual feedback
    setMods(mods.filter((m) => m.name !== mod.name));
  };

  return (
    <div className="p-6">
      <div className="flex justify-between items-center mb-6">
        <h2 className="text-2xl font-bold">{t('mod_manager')}</h2>
        <div className="flex items-center">
          <input
            type="text"
            placeholder={t('mod_search')}
            className="w-64 p-2 bg-gray-800 border border-gray-700 rounded-sm"
          />
          <button
            onClick={() => setAddModPopupOpen(true)}
            className="ml-2 p-2 bg-indigo-500 hover:bg-indigo-600 rounded-sm text-white"
          >
            <Plus size={20} />
          </button>
        </div>
      </div>

      <div className="space-y-3">
        {mods.map((mod, index) => (
          <div
            key={index}
            className="bg-gray-800 p-4 rounded-sm flex justify-between items-center"
          >
            <div>
              <h3 className="font-semibold">{mod.name}</h3>
              <p className="text-sm text-gray-400">{mod.description}</p>
            </div>
            <div className="flex items-center">
              <span className="text-xs text-indigo-400 mr-3">
                {mod.version}
              </span>
              <label
                htmlFor={`mod-toggle-${index}`}
                className="flex items-center cursor-pointer"
              >
                <div className="relative">
                  <input
                    type="checkbox"
                    id={`mod-toggle-${index}`}
                    className="sr-only"
                    checked={mod.enabled}
                    onChange={(e) => handleToggleMod(mod, e.target.checked)}
                  />
                  <div className="block bg-gray-600 w-14 h-8 rounded-full"></div>
                  <div className="dot absolute left-1 top-1 bg-white w-6 h-6 rounded-full transition-transform"></div>
                </div>
              </label>
              <button
                onClick={() => handleDeleteMod(mod)}
                className="ml-4 p-2 text-red-500 hover:text-red-400"
              >
                <Trash2 size={20} />
              </button>
            </div>
          </div>
        ))}
      </div>
      <AddModPopup
        isOpen={isAddModPopupOpen}
        onClose={() => {
          setMods([]);
          setAddModPopupOpen(false);
        }}
      />
    </div>
  );
}

function RamUsageBar({ totalRam, ramUsage, setRamUsage }) {
  const { t } = useTranslation();
  const ramUsageGB = (ramUsage / 1024).toFixed(1);

  const handleRamChange = (event) => {
    const newRamValue = parseInt(event.target.value, 10);
    setRamUsage(newRamValue);
  };

  const handleRamChangeEnd = () => {
    invoke('set_ram_usage', { ramUsage: ramUsage }).catch(console.error);
  };

  return (
    <div className="bg-gray-800 p-6 rounded-sm">
      <h3 className="text-lg font-semibold mb-1">{t('mem_alloc_title')}</h3>
      <p className="text-sm text-gray-400 mb-4">{t('mem_alloc_description')}</p>
      <div className="flex items-center">
        <input
          type="range"
          min="1.0"
          max={totalRam > 1024 ? Math.round((totalRam * 10) / 1024) / 10 : 8}
          step="0.5"
          value={ramUsage}
          onInput={handleRamChange}
          onMouseUp={handleRamChangeEnd}
          className="w-64"
        />
        <data
          id="ram_usage_label"
          className="ml-4 tabular-nums"
          value={ramUsageGB}
        >
          {ramUsageGB} GB
        </data>
      </div>
    </div>
  );
}

function SettingsTab() {
  const { t } = useTranslation();

  const [totalRam, setTotalRam] = useState(0);
  const [ramUsage, setRamUsage] = useState(2048);

  useEffect(() => {
    invoke('get_total_ram').then(setTotalRam).catch(console.error);
    invoke('get_ram_usage')
      .then((ram) => {
        if (ram) setRamUsage(ram);
      })
      .catch(console.error);
  }, []);

  function saveSettings() {
    invoke('set_ram_usage', { ramUsage: ramUsage }).catch(console.error);
    invoke('save').catch((e) => console.error("Couldn't save settings.", e));
    console.log('Settings Saved!');
  }

  return (
    <div className="p-6">
      <h2 className="text-2xl font-bold mb-6">{t('settings_tab')}</h2>

      <div className="space-y-6">
        <RamUsageBar
          totalRam={totalRam}
          ramUsage={ramUsage}
          setRamUsage={setRamUsage}
        />

        {/*<div className="bg-gray-800 p-6 rounded-sm">*/}
        {/* <h3 className="text-lg font-semibold mb-1">{t("launch_options_title")}</h3>*/}
        {/* <p className="text-sm text-gray-400 mb-4">{t("launch_options_description")}</p>*/}
        {/* <div className="space-y-3">*/}
        {/* <div className="flex items-center">*/}
        {/* <input type="checkbox" id="fullscreen" className="mr-2"/>*/}
        {/* <label htmlFor="fullscreen">{t("lo_fullscreen")}</label>*/}
        {/* </div>*/}
        {/* <div className="flex items-center">*/}
        {/* <input type="checkbox" id="close-launcher" className="mr-2"/>*/}
        {/* <label htmlFor="close-launcher">{t("lo_close_on_start")}</label>*/}
        {/* </div>*/}
        {/* <div className="flex items-center">*/}
        {/* <input type="checkbox" id="check-updates" className="mr-2"/>*/}
        {/* <label htmlFor="check-updates">{t("startup_update_check")}</label>*/}
        {/* </div>*/}
        {/* </div>*/}
        {/*</div>*/}

        <div className="flex justify-end">
          <Button onClick={saveSettings}>{t('save_settings')}</Button>
        </div>
      </div>
    </div>
  );
}
