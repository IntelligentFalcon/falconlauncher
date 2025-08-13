import {useEffect, useState} from 'react';
import {Settings, Package, Home, Play} from 'lucide-react';
import {invoke} from "@tauri-apps/api/core";
import {listen} from '@tauri-apps/api/event';
import LoginPopup from './LoginPopup';
import {t, loadLanguage} from "./i18n";


export default function FalconClient() {
    const [activeTab, setActiveTab] = useState("home");
    const [downloadProgress, setDownloadProgress] = useState(0);
    const [isDownloading, setIsDownloading] = useState(false);
    const [isShowingSnapshots, setShowingSnapshots] = useState(false);
    const [isShowingAlpha, setShowingAlpha] = useState(false);
    const [versions, setVersions] = useState([]);
    const [selectedVersion, setSelectedVersion] = useState("");
    const [username, setUsername] = useState("");
    const [statusMessage, setStatusMessage] = useState('Ready to play');
    const [isLoginPopupOpen, setIsLoginPopupPopupOpen] = useState(false);
    const [profiles, setProfiles] = useState([])
    const [currentLanguage, setCurrentLanguage] = useState("fa");

    useEffect(() => {
        invoke("get_language").then(lang => setCurrentLanguage(lang)).catch("Failed to change the default language");
        loadLanguage(currentLanguage).catch(console.error);
    }, []);

    async function load_versions() {
        invoke("get_versions")
            .then((v) => {
                setVersions(v);
                if (v.length > 0) {
                    setSelectedVersion(v[0]);
                }
            })
            .catch((e) => console.error("Failed to fetch versions:", e));
    }

    useEffect(() => {
        invoke("get_profiles").then((v) => {
            setProfiles(v);
            if (profiles.length > 0) {
                setUsername(v[0]);
            }
        });
    })

    useEffect(() => {
        load_versions().catch((e) => console.error("Initial version load failed!", e));
        invoke("get_username").then(setUsername).catch(() => console.error("Couldn't get the username"));

        async function registerEvents() {
            const unlistenProgress = await listen('progress', (event) => {
                console.log('Progress:', event.payload);
                setStatusMessage(event.payload);
            });
            const unlistenProgressBar = await listen('progressBar', (event) => {
                console.log('Progress Bar:', event.payload);
                if (event.payload >= 100) {
                    setIsDownloading(false);
                }
                setDownloadProgress(event.payload);
            });
            // Cleanup on component unmount
            return () => {
                unlistenProgress();
                unlistenProgressBar();
            };
        }

        registerEvents().catch((e) => console.error("Failed to register events", e));
    }, []);


    const handlePlay = async () => {
        if (!selectedVersion && versions.length > 0) {
            setSelectedVersion(versions[0]);
        }
        setIsDownloading(true);
        try {
            await invoke("set_username", {username});
            await invoke("save");
            await invoke("play_button_handler", {selectedVersion});
        } catch (e) {
            console.error("Failed to launch game:", e);
            setIsDownloading(false);
        }

        if (downloadProgress >= 100) {
            setIsDownloading(false);
            setDownloadProgress(0);
        }
    };

    const toggleShowingSnapshots = () => {
        const newValue = !isShowingSnapshots;
        setShowingSnapshots(newValue);
        invoke("set_allow_snapshot", {enabled: newValue})
            .then(() => invoke("reload_versions"))
            .then(() => load_versions())
            .catch(e => console.error("Failed to toggle snapshots and reload", e));
    };

    const toggleShowAlpha = () => {
        const newValue = !isShowingAlpha;
        setShowingAlpha(newValue);
        invoke("set_allow_old_versions", {enabled: newValue})
            .then(() => invoke("reload_versions"))
            .then(() => load_versions())
            .catch(e => console.error("Failed to toggle old versions and reload", e));
    };

    // New function to handle language change
    const handleLanguageChange = async (lang) => {
        setCurrentLanguage(lang);
        invoke("set_language", {lang: lang}).catch("Failed to change language!")
        await loadLanguage(lang);
    };


    return (<div className="flex flex-col w-full h-screen bg-gray-900 text-gray-200 overflow-hidden">
            {/* Header */}
            <div className="flex justify-between items-center px-4 sm:px-6 py-3 bg-gray-800 border-b border-gray-700">
                <div className="flex items-center flex-wrap gap-2">
                    <h1 className="text-lg sm:text-xl font-bold text-indigo-400">{t("app_name")}</h1>
                    <span className="text-xs text-gray-400">v1.0.0</span>
                </div>
                {/* Language Change Button */}
                <div className="flex items-center">
                    <button
                        className="px-3 py-1 bg-yellow-400 hover:bg-yellow-500 text-white font-bold rounded text-sm"
                        onClick={() => handleLanguageChange(currentLanguage === 'fa' ? 'en' : 'fa')}
                    >
                        {currentLanguage === 'fa' ? 'English' : 'فارسی'}
                    </button>
                </div>
            </div>

            <div className="flex flex-1 flex-col lg:flex-row overflow-hidden">
                {/* Sidebar */}
                <div className="w-full lg:w-64 md:w-48 bg-gray-800 flex flex-col">
                    {/* Profile selection */}
                    <div className="p-4 sm:p-6 flex flex-col items-center">
                        <h2 className="text-base sm:text-lg font-semibold mb-4">{t("select_profile")}</h2>
                        <select
                            name="Profile"
                            className="w-full mb-2 p-2 bg-gray-900 border border-indigo-500 rounded text-gray-200 focus:outline-none text-sm sm:text-base"
                            onChange={(e) => setUsername(e.target.value)}
                        >
                            {profiles.map((v) => <option key={v} value={v}>{v}</option>)}
                        </select>
                        <button
                            className="w-full mb-2 p-2 bg-gray-900 border border-indigo-500 rounded text-gray-200 focus:outline-none text-sm sm:text-base"
                            onClick={() => setIsLoginPopupPopupOpen(true)}
                        >
                            {t("create_profile")}
                        </button>
                    </div>

                    {/* Version selection */}
                    <div className="px-4 sm:px-6 pb-4">
                        <label className="block text-sm font-semibold mb-2">{t("game_version")}</label>
                        <select
                            className="w-full p-2 bg-gray-900 border border-gray-700 rounded text-gray-200 text-sm sm:text-base"
                            value={selectedVersion}
                            onChange={(e) => setSelectedVersion(e.target.value)}
                        >
                            {versions.map((version) =>
                                <option key={version} id={version}>{version}</option>
                            )}
                        </select>
                        <div className="flex items-center justify-left mt-6">
                            <button
                                onClick={toggleShowingSnapshots}
                                className={`relative inline-flex h-6 w-10 items-center rounded-full transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${isShowingSnapshots ? 'bg-blue-600' : 'bg-gray-300'}`}
                                role="switch"
                                aria-checked={isShowingSnapshots}
                            >
                            <span
                                className={`inline-block h-4 w-4 transform rounded-full bg-white shadow-lg transition-transform duration-200 ease-in-out ${isShowingSnapshots ? 'translate-x-5' : 'translate-x-1'}`}
                            />
                            </button>
                            <label className='ml-2'>{t("show_snapshots")}</label>
                        </div>

                        <div className="flex items-center justify-left mt-2">
                            <button
                                onClick={toggleShowAlpha}
                                className={`relative inline-flex h-6 w-10 items-center rounded-full transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${isShowingAlpha ? 'bg-blue-600' : 'bg-gray-300'}`}
                                role="switch"
                                aria-checked={isShowingAlpha}
                            >
                            <span
                                className={`inline-block h-4 w-4 transform rounded-full bg-white shadow-lg transition-transform duration-200 ease-in-out ${isShowingAlpha ? 'translate-x-5' : 'translate-x-1'}`}
                            />
                            </button>
                            <label className='ml-2'>{t("show_old_versions")}</label>
                        </div>
                    </div>


                    {/* Navigation */}
                    <div className="flex-1 py-4">
                        <NavItem
                            icon={<Home size={18}/>}
                            title={t("home_tab")}
                            active={activeTab === 'home'}
                            onClick={() => setActiveTab('home')}
                        />
                        <NavItem
                            icon={<Package size={18}/>}
                            title={t("mods_tab")}
                            active={activeTab === 'mods'}
                            onClick={() => setActiveTab('mods')}
                        />
                        <NavItem
                            icon={<Settings size={18}/>}
                            title={t("settings_tab")}
                            active={activeTab === 'settings'}
                            onClick={() => setActiveTab('settings')}
                        />
                    </div>

                    {/* Play button */}
                    <div className="p-4 sm:p-6 border-t border-gray-700">
                        <button
                            disabled={isDownloading}
                            className="w-full py-2 sm:py-3 bg-green-600 hover:bg-green-700 text-white font-bold rounded flex items-center justify-center disabled:bg-gray-500 text-sm sm:text-base"
                            onClick={handlePlay}
                        >
                            <Play size={18} className="mr-2"/>
                            {t("play")}
                        </button>
                        {isDownloading && (
                            <div className="w-full bg-gray-700 rounded-full h-2 mt-4">
                                <div
                                    className="bg-indigo-500 h-2 rounded-full"
                                    style={{width: `${downloadProgress}%`}}
                                ></div>
                            </div>
                        )}
                        <p className="text-xs mt-2 text-gray-400">{statusMessage}</p>
                    </div>
                </div>

                {/* Main content */}
                <div className="flex-1 overflow-auto p-4 sm:p-6">
                    {activeTab === 'home' && <HomeTab/>}
                    {activeTab === 'settings' && <SettingsTab/>}
                    {activeTab === 'mods' && <ModsTab/>}
                </div>
            </div>

            <LoginPopup isOpen={isLoginPopupOpen} onClose={() => setIsLoginPopupPopupOpen(false)}/>
        </div>
    );
}

function NavItem({icon, title, active, onClick}) {
    return (<div
        className={`flex items-center px-6 py-3 cursor-pointer ${active ? 'bg-gray-700 border-l-4 border-indigo-500' : 'hover:bg-gray-700'}`}
        onClick={onClick}
    >
        <div className={`mr-3 ${active ? 'text-indigo-400' : 'text-gray-400'}`}>
            {icon}
        </div>
        <span className={active ? 'font-semibold' : ''}>{title}</span>
    </div>);
}

function HomeTab() {
    const newsArticles = [{
        title: 'Minecraft 1.20.4 Released',
        content: 'The latest version brings new features and bug fixes',
        date: '3 days ago'
    }, {
        title: 'Community Event: Building Competition',
        content: 'Join our weekly building competition',
        date: '1 week ago'
    }, {
        title: 'New Mod Spotlight: Enhanced Biomes',
        content: 'Discover incredible new biomes with this mod',
        date: '2 weeks ago'
    }];

    return (<div className="p-8">
        <h2 className="text-2xl font-bold mb-6">{t("minecraft_news")}</h2>

        <div className="space-y-4">
            {newsArticles.map((article, index) => (<div key={index} className="bg-gray-800 p-6 rounded">
                <h3 className="text-xl font-semibold mb-2">{article.title}</h3>
                <p className="text-gray-300 mb-3">{article.content}</p>
                <p className="text-sm text-indigo-400 italic">{article.date}</p>
            </div>))}
        </div>
    </div>);
}

function ModsTab() {
    const mods = [{
        name: 'Optifine', description: 'Optimize performance and add features', version: '1.20.4'
    }, {name: 'JEI (Just Enough Items)', description: 'View all items and recipes', version: '1.20.4'}, {
        name: 'Sodium', description: 'Performance optimization', version: '1.20.4'
    }, {name: 'Fabric API', description: 'Core API for Fabric mods', version: '1.20.4'}, {
        name: 'Litematica', description: 'Schematic mod for building', version: '1.20.4'
    },];

    return (<div className="p-6">
        <div className="flex justify-between items-center mb-6">
            <h2 className="2xl font-bold">{t("mod_manager")}</h2>
            <input
                type="text"
                placeholder={t("mod_search")}
                className="w-64 p-2 bg-gray-800 border border-gray-700 rounded"
            />
        </div>

        <div className="space-y-3">
            {mods.map((mod, index) => (
                <div key={index} className="bg-gray-800 p-4 rounded flex justify-between items-center">
                    <div>
                        <h3 className="font-semibold">{mod.name}</h3>
                        <p className="text-sm text-gray-400">{mod.description}</p>
                    </div>
                    <div className="flex items-center">
                        <span className="text-xs text-indigo-400 mr-3">{mod.version}</span>
                        <button className="px-3 py-1 bg-indigo-500 hover:bg-indigo-600 rounded text-sm">
                            Install
                        </button>
                    </div>
                </div>))}
        </div>
    </div>);
}

function RamUsageBar({totalRam}) {
    const [ramUsage, setRamUsage] = useState(0);
    const [ramUsagePretiffied, setRamUsagePrettified] = useState("2GB");
    if (ramUsage === 0) invoke("get_ram_usage")
        .then(ramUsage => {
            setRamUsage(ramUsage);
            setRamUsagePrettified((ramUsage / 1024).toFixed(1) + " GB");
        })
        .catch("Not working fuck");

    return <div className="bg-gray-800 p-6 rounded">
        <h3 className="text-lg font-semibold mb-1">{t("mem_alloc_title")}</h3>
        <p className="text-sm text-gray-400 mb-4">{t("mem_alloc_description")}</p>
        <div className="flex items-center">
            <input type="range" min="1.0" max={parseInt(totalRam / 1024)} value={ramUsage}
                   onInput={event => {
                       setRamUsage(event.target.value);
                       setRamUsagePrettified((event.target.value / 1024).toFixed(1) + " GB");
                       invoke("set_ram_usage", {ramUsage: event.target.value}).catch("").then();
                   }} className="w-64"/>

            <data id="ram_usage_label" className="ml-4"
                  value={ramUsagePretiffied}>{ramUsagePretiffied}</data>
        </div>
    </div>
}

function SettingsTab() {
    const [totalRam, setTotalRam] = useState(0)
    if (totalRam === 0) invoke("get_total_ram").catch((e) => console.error("I hate things not to work", e)).then(ram => setTotalRam(ram))

    function save() {
        invoke("set_ram_usage", {ramUsage: gRamUsage}).catch("Aw man you screwed it up");
        invoke("save").catch("Couldn't save file.")
    }

    return (<div className="p-6">
        <h2 className="text-2xl font-bold mb-6">Settings</h2>

        <div className="space-y-6">
            {/* Memory Settings */}
            <RamUsageBar totalRam={totalRam}/>

            {/* Launch Options */}
            <div className="bg-gray-800 p-6 rounded">
                <h3 className="text-lg font-semibold mb-1">{t("launch_options_title")}</h3>
                <p className="text-sm text-gray-400 mb-4">{t("launch_options_description")}</p>

                <div className="space-y-3">
                    <div className="flex items-center">
                        <input type="checkbox" id="fullscreen" className="mr-2"/>
                        <label htmlFor="fullscreen">{t("lo_fullscreen")}</label>
                    </div>
                    <div className="flex items-center">
                        <input type="checkbox" id="close-launcher" className="mr-2"/>
                        <label htmlFor="close-launcher">{t("lo_close_on_start")}</label>
                    </div>
                    <div className="flex items-center">
                        <input type="checkbox" id="check-updates" className="mr-2"/>
                        <label htmlFor="check-updates">{t("startup_update_check")}</label>
                    </div>
                </div>
            </div>

            <div className="flex justify-end">
                <button className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded" onClick={save}>
                    Save Settings
                </button>
            </div>
        </div>
    </div>);
}