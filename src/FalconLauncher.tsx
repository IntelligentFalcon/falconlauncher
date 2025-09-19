import {useCallback, useEffect, useState} from 'react';
import {Grid, Home, List, Package, Play, Settings, X, Plus, Trash2} from 'lucide-react';
import {invoke} from "@tauri-apps/api/core";
import {listen} from '@tauri-apps/api/event';
import LoginPopup from './LoginPopup';
import {getCurrentLang, loadLanguage, t} from "./i18n";
import {publicDir} from "@tauri-apps/api/path";

const FlagIran = ({className}) => (
    <svg viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg" aria-hidden="true" role="img" className={className}
         preserveAspectRatio="xMidYMid meet">
        <path fill="#DA0001" d="M0 27a4 4 0 0 0 4 4h28a4 4 0 0 0 4-4v-4H0v4z"></path>
        <path fill="#EEE" d="M0 13h36v10H0z"></path>
        <path fill="#239F40" d="M36 13V9a4 4 0 0 0-4-4H4a4 4 0 0 0-4 4v4h36z"></path>
        <path fill="#E96667" d="M0 23h36v1H0z"></path>
        <g fill="#BE1931">
            <path
                d="M19.465 14.969c.957.49 3.038 2.953.798 5.731c1.391-.308 3.162-4.408-.798-5.731zm-2.937 0c-3.959 1.323-2.189 5.423-.798 5.731c-2.24-2.778-.159-5.241.798-5.731zm1.453-.143c.04.197 1.101.436.974-.573c-.168.408-.654.396-.968.207c-.432.241-.835.182-.988-.227c-.148.754.587.975.982.593z"></path>
            <path
                d="M20.538 17.904c-.015-1.248-.677-2.352-1.329-2.799c.43.527 1.752 3.436-.785 5.351l.047-5.097l-.475-.418l-.475.398l.08 5.146l-.018-.015c-2.563-1.914-1.233-4.837-.802-5.365c-.652.447-1.315 1.551-1.329 2.799c-.013 1.071.477 2.243 1.834 3.205a6.375 6.375 0 0 1-1.678.201c.464.253 1.34.192 2.007.131l.001.068l.398.437l.4-.455v-.052c.672.062 1.567.129 2.039-.128a6.302 6.302 0 0 1-1.732-.213c1.344-.961 1.83-2.127 1.817-3.194z"></path>
        </g>
        <path fill="#7BC58C" d="M0 12h36v1H0z"></path>
    </svg>);

// === UPDATED FLAG COMPONENT (UK) ===
const FlagUK = ({className}) => (
    <svg height="200px" width="200px" version="1.1" id="Layer_1" xmlns="http://www.w3.org/2000/svg"
         xmlnsXlink="http://www.w3.org/1999/xlink" viewBox="0 0 512 512" xmlSpace="preserve" className={className}>
        <path style={{fill: "#41479B"}}
              d="M473.655,88.276H38.345C17.167,88.276,0,105.443,0,126.621V385.38 c0,21.177,17.167,38.345,38.345,38.345h435.31c21.177,0,38.345-17.167,38.345-38.345V126.621 C512,105.443,494.833,88.276,473.655,88.276z"></path>
        <path style={{fill: "#F5F5F5"}}
              d="M511.469,120.282c-3.022-18.159-18.797-32.007-37.814-32.007h-9.977l-163.54,107.147V88.276h-88.276 v107.147L48.322,88.276h-9.977c-19.017,0-34.792,13.847-37.814,32.007l139.778,91.58H0v88.276h140.309L0.531,391.717 c3.022,18.159,18.797,32.007,37.814,32.007h9.977l163.54-107.147v107.147h88.276V316.577l163.54,107.147h9.977 c19.017,0,34.792-13.847,37.814-32.007l-139.778-91.58H512v-88.276H371.691L511.469,120.282z"></path>
        <g>
            <polygon style={{fill: "#FF4B55"}}
                     points="282.483,88.276 229.517,88.276 229.517,229.517 0,229.517 0,282.483 229.517,282.483 229.517,423.724 282.483,423.724 282.483,282.483 512,282.483 512,229.517 282.483,229.517 "></polygon>
            <path style={{fill: "#FF4B55"}}
                  d="M24.793,421.252l186.583-121.114h-32.428L9.224,410.31 C13.377,415.157,18.714,418.955,24.793,421.252z"></path>
            <path style={{fill: "#FF4B55"}}
                  d="M346.388,300.138H313.96l180.716,117.305c5.057-3.321,9.277-7.807,12.287-13.075L346.388,300.138z"></path>
            <path style={{fill: "#FF4B55"}}
                  d="M4.049,109.475l157.73,102.387h32.428L15.475,95.842C10.676,99.414,6.749,104.084,4.049,109.475z"></path>
            <path style={{fill: "#FF4B55"}}
                  d="M332.566,211.862l170.035-110.375c-4.199-4.831-9.578-8.607-15.699-10.86L300.138,211.862H332.566z"></path>
        </g>
    </svg>);


function VersionSelectorPopup({isOpen, onClose, onVersionSelect, currentLanguage = 'fa'}) {
    const [viewMode, setViewMode] = useState('grid');
    const [activeMajor, setActiveMajor] = useState('1.21');
    const [activeSpecific, setActiveSpecific] = useState(null);
    const [showForge, setShowForge] = useState(false);
    const [showNeoForge, setShowNeoForge] = useState(false);
    const [showLiteLoader, setShowLiteLoader] = useState(false);
    const [showFabric, setShowFabric] = useState(false);
    const [versionsData, setVersionsData] = useState({});

    const updateVersions = (forge, fabric, neoforge, liteloader) => {
        invoke("load_categorized_versions", {
            fabric: fabric,
            forge: forge,
            neoForge: neoforge,
            liteLoader: liteloader
        })
            .then(categories => {
                const v = {};
                for (const category of categories) {
                    v[category.name] = category.versions.map(x => ({v: x.id, d: x.date, base: x.base}));

                }
                setVersionsData(v);
            })
            .catch(err => console.error("Error loading versions:", err));
    }

    if (Object.keys(versionsData).length === 0) {
        updateVersions(showForge, showFabric, showNeoForge, showLiteLoader);
    }
    const majorVersions = Object.keys(versionsData);
    useEffect(() => {

        if (versionsData[activeMajor] && versionsData[activeMajor].length > 0) {
            setActiveSpecific(versionsData[activeMajor][0]);
        }
    }, [activeMajor]);

    if (!isOpen) return null;

    const handleInstall = () => {
        if (activeSpecific) {
            onVersionSelect(activeSpecific);
        }
        onClose();
    };

    const SpecificVersionItem = ({version, date, type}) => {
        const isActive = activeSpecific === version;
        const baseClasses = "cursor-pointer transition-all duration-200 ease-in-out rounded-lg";
        const activeClasses = "bg-indigo-600 border-indigo-400 text-white";
        const hoverClasses = "hover:bg-zinc-700";

        if (type === 'grid') {
            return (<div onClick={() => setActiveSpecific(version)}
                         className={`${baseClasses} p-4 text-center border ${isActive ? activeClasses : 'bg-zinc-800 border-zinc-700 ' + hoverClasses}`}>
                <div className="font-bold text-lg">{version.v}</div>
                <div className={`text-sm ${isActive ? 'text-gray-200' : 'text-zinc-400'}`}>{date}</div>
            </div>);
        }
        return (<div onClick={() => setActiveSpecific(version)}
                     className={`${baseClasses} flex justify-between items-center p-4 border border-transparent ${isActive ? activeClasses : hoverClasses}`}>
            <div className="font-bold">{version.v}</div>
            <div className={`text-sm ${isActive ? 'text-gray-200' : 'text-zinc-400'}`}>{date}</div>
        </div>);
    };

    const directionClass = currentLanguage === 'fa' ? 'rtl' : 'ltr';
    const fontClass = currentLanguage === 'fa' ? 'font-vazir' : 'font-inter';

    return (<>
        <style>{`
                @import url('https://fonts.googleapis.com/css2?family=Vazirmatn:wght@400;500;700&display=swap');
                @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;700&display=swap');
                .font-vazir { font-family: 'Vazirmatn', sans-serif; }
                .font-inter { font-family: 'Inter', sans-serif; }
            `}</style>
        <div
            className={`fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50 transition-opacity duration-300 ${isOpen ? 'opacity-100' : 'opacity-0'} ${fontClass}`}
            dir={directionClass}
        >
            <div
                className={`w-[1200px] h-[750px] max-w-[95vw] max-h-[90vh] bg-zinc-900 rounded-lg flex overflow-hidden border border-zinc-700 shadow-2xl text-gray-200 transition-transform duration-300 ${isOpen ? 'scale-100' : 'scale-95'}`}>
                <aside className="w-[280px] bg-gray-800 p-6 border-l border-zinc-700 flex flex-col">
                    <h2 className="text-xl font-bold mb-6">{t('mod_loaders', currentLanguage)}</h2>
                    <div className="space-y-3">
                        <div className="flex items-center bg-zinc-700 p-3 rounded-md">
                            <input type="checkbox" id="forge" className="w-5 h-5 accent-indigo-500 cursor-pointer"
                                   checked={showForge} onChange={e => {
                                // HERE
                                setShowForge(prev => {
                                    const newValue = !prev;
                                    updateVersions(newValue, showFabric, showNeoForge, showLiteLoader);

                                    return newValue;
                                });
                            }}/>
                            <label htmlFor="forge"
                                   className="mx-3 text-base cursor-pointer grow">{t('install_forge', currentLanguage)}</label>
                        </div>
                        <div className="flex items-center bg-zinc-700 p-3 rounded-md">
                            <input type="checkbox" id="fabric"
                                   className="w-5 h-5 accent-indigo-500 cursor-pointer" onChange={e => {
                                // HERE
                                setShowFabric(prev => {
                                    const newValue = !prev;
                                    updateVersions(showForge, newValue, showNeoForge, showLiteLoader);

                                    return newValue;
                                });
                            }}/>
                            <label htmlFor="fabric"
                                   className="mx-3 text-base cursor-pointer grow">{t('install_fabric', currentLanguage)}</label>
                        </div>
                        <div className="flex items-center bg-zinc-700 p-3 rounded-md">
                            <input type="checkbox" id="liteloader"
                                   className="w-5 h-5 accent-indigo-500 cursor-pointer"/>
                            <label htmlFor="liteloader"
                                   className="mx-3 text-base cursor-pointer grow">{t('install_liteloader', currentLanguage)}</label>
                        </div>
                        <div className="flex items-center bg-zinc-700 p-3 rounded-md">
                            <input type="checkbox" id="neoforge"
                                   className="w-5 h-5 accent-indigo-500 cursor-pointer"/>
                            <label htmlFor="neoforge"
                                   className="mx-3 text-base cursor-pointer grow">{t('install_neoforge', currentLanguage)}</label>
                        </div>
                    </div>
                    <button onClick={handleInstall}
                            className="w-full mt-auto bg-green-600 hover:bg-green-700 text-white font-bold py-3 rounded-lg transition-colors">
                        {t('install_selected', currentLanguage)}
                    </button>
                </aside>
                <main className="grow p-6 flex flex-col overflow-y-auto">
                    <header className="flex justify-between items-center mb-6">
                        <h1 className="text-3xl font-bold">{t('minecraft_version', currentLanguage)} {activeMajor}</h1>
                        <div className="flex space-x-2">
                            <button onClick={() => setViewMode('grid')}
                                    className={`p-2 rounded-md transition-colors ${viewMode === 'grid' ? 'bg-indigo-600 text-white' : 'bg-zinc-700 text-zinc-400 hover:bg-zinc-600'}`}>
                                <Grid size={20}/>
                            </button>
                            <button onClick={() => setViewMode('list')}
                                    className={`p-2 rounded-md transition-colors ${viewMode === 'list' ? 'bg-indigo-600 text-white' : 'bg-zinc-700 text-zinc-400 hover:bg-zinc-600'}`}>
                                <List size={20}/>
                            </button>
                        </div>
                    </header>
                    {!versionsData[activeMajor] || versionsData[activeMajor].length === 0 ? (
                        <p>Loading versions...</p>
                    ) : viewMode === 'grid' ? (
                        <div className="grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-4">
                            {versionsData[activeMajor].map(item => (
                                <SpecificVersionItem
                                    key={item.v}
                                    version={item}
                                    date={item.d}
                                    type="grid"
                                />
                            ))}
                        </div>
                    ) : (
                        <div className="flex flex-col space-y-2">
                            {versionsData[activeMajor].map(item => (
                                <SpecificVersionItem
                                    key={item.v}
                                    version={item}
                                    date={item.d}
                                    type="list"
                                />
                            ))}
                        </div>
                    )}

                </main>
                <aside className="w-[120px] bg-gray-800 p-2 border-r border-zinc-700 overflow-y-auto">
                    <ul className="space-y-1">
                        {majorVersions.map(v => (<li key={v} onClick={() => setActiveMajor(v)}
                                                     className={`px-3 py-4 text-center font-bold text-lg rounded-md cursor-pointer transition-colors ${activeMajor === v ? 'bg-zinc-900 text-white' : 'text-zinc-400 hover:bg-zinc-700'}`}>
                            {v}
                        </li>))}
                    </ul>
                </aside>
                <button onClick={onClose}
                        className="absolute top-4 right-4 text-gray-400 hover:text-white transition-colors">
                    <X size={24}/>
                </button>
            </div>
        </div>
    </>);
}


export default function FalconLauncher() {
    const [activeTab, setActiveTab] = useState("home");
    const [downloadProgress, setDownloadProgress] = useState(0);
    const [isDownloading, setIsDownloading] = useState(false);
    const [versions, setVersions] = useState([]);
    const [selectedVersion, setSelectedVersion] = useState("");
    const [username, setUsername] = useState("");
    const [statusMessage, setStatusMessage] = useState('Ready to play');
    const [isLoginPopupOpen, setIsLoginPopupPopupOpen] = useState(false);
    const [isVersionSelectorOpen, setIsVersionSelectorOpen] = useState(false);
    const [profiles, setProfiles] = useState([]);
    const [currentLanguage, setCurrentLanguage] = useState("fa");

    const loadVersions = useCallback(async () => {
        try {
            const v = await invoke("get_versions");
            setVersions(v);
            if (v.length > 0 && !selectedVersion) {
                setSelectedVersion(v[0]);
            }
        } catch (e) {
            console.error("Failed to fetch versions:", e);
        }
    }, [selectedVersion]);

    function reloadProfiles() {
        useEffect(() => {
            invoke("get_profiles").then((v) => {
                setProfiles(v);
                if (v.length > 0) {
                    invoke("get_username").then(setUsername).catch(console.error);
                }
            }).catch(console.error);
        }, [profiles, username]);
    }

    function reloadLanguage() {
        useEffect(() => {
            invoke("get_language").then(lang => {
                setCurrentLanguage(lang);
                loadLanguage(lang).catch("Failed to load the language");
            }).catch(console.error);
        }, [currentLanguage]);
    }

    reloadLanguage();
    reloadProfiles();

    useEffect(() => {
        // // Mock translations for popup
        // const newTranslations = {
        //     en: { ...t.en, install_new_version: 'Install New Version', mod_loaders: 'Mod Loaders', install_selected: 'Install Selected', minecraft_version: 'Minecraft', install_forge: 'Install Forge', install_fabric: 'Install Fabric', install_liteloader: 'Install LiteLoader', install_neoforge: 'Install NeoForge' },
        //     fa: { ...t.fa, install_new_version: 'نصب نسخه جدید', mod_loaders: 'ماد لودرها', install_selected: 'نصب نسخه انتخابی', minecraft_version: 'ماینکرفت', install_forge: 'نصب Forge', install_fabric: 'نصب Fabric', install_liteloader: 'نصب LiteLoader', install_neoforge: 'نصب NeoForge' }
        // };
        // Object.assign(t, newTranslations);


        loadVersions().then(() => console.log("loaded versions!")).catch("Error!");

        let unlistenProgress, unlistenProgressBar;
        const registerEvents = async () => {
            unlistenProgress = await listen('progress', (event) => setStatusMessage(event.payload));
            unlistenProgressBar = await listen('progressBar', (event) => {
                if (event.payload >= 100) setIsDownloading(false);
                setDownloadProgress(event.payload);
            });
        };

        registerEvents().then(() => console.log("Done!"));

        return () => {
            unlistenProgress?.();
            unlistenProgressBar?.();
        };
    }, [loadVersions]);

    useEffect(() => {
        document.body.className = '';
        document.body.classList.add(`lang-${currentLanguage}`);
    }, [currentLanguage]);

    const handlePlay = async () => {
        if (!selectedVersion && versions.length > 0) setSelectedVersion(versions[0]);
        setIsDownloading(true);
        try {
            await invoke("set_username", {username: username});
            await invoke("save");
            await invoke("play_button_handler", {selectedVersion: selectedVersion});
        } catch (e) {
            console.error("Failed to launch game:", e);
            setIsDownloading(false);
        }
    };

    const handleLanguageChange = async (lang) => {
        setCurrentLanguage(lang);
        await invoke("set_language", {lang});
        await invoke("save");
        await loadLanguage(lang).catch(console.error);
    };

    return (<div className="flex flex-col w-full h-screen bg-gray-900 text-gray-200 overflow-hidden">
        <div className="flex justify-between items-center px-4 sm:px-6 py-3 bg-gray-800 border-b border-gray-700">
            <div className="flex items-center flex-wrap gap-2">
                <h1 className="text-lg sm:text-xl font-bold text-indigo-400">{t("app_name")}</h1>
                <span className="text-xs text-gray-400">v1.0.0</span>
            </div>
            <div className="flex items-center">
                <button
                    className="p-1 rounded-full hover:bg-gray-700 transition-colors"
                    onClick={() => handleLanguageChange(currentLanguage === 'fa' ? 'en' : 'fa')}
                    title="Change Language"
                >
                    {currentLanguage === "fa" ? <FlagUK className="w-8 h-6 rounded-xs"/> :
                        <FlagIran className="w-8 h-6 rounded-xs"/>}
                </button>
            </div>
        </div>

        <div className="flex flex-1 flex-col lg:flex-row overflow-hidden">
            <div className="w-full lg:w-64 md:w-48 bg-gray-800 flex flex-col">

                <div className="p-4 sm:p-6 flex flex-col">
                    {/*WEIRD ISSUE HERE Background gray color is not working*/}
                    <select
                        name="Profile"
                        className="az-select az-bg-gray-900 az-w-full az-mb-2 az-p-2 az-border az-border-indigo-500 az-rounded az-text-gray-200 az-focus:az-outline-none az-text-sm sm:az-text-base"
                        value={username}
                        onChange={async (e) => {
                            setUsername(e.target.value);
                            await invoke("set_username", {username: e.target.value});
                        }}
                    >
                        <option value="" disabled>{t("select_profile")}</option>
                        {profiles.map((v) => <option key={v} value={v}>{v}</option>)}
                    </select>

                    <button
                        className="w-full mb-4 p-2 bg-gray-900 border border-indigo-500 rounded-sm text-gray-200 focus:outline-hidden text-sm sm:text-base"
                        onClick={() => setIsLoginPopupPopupOpen(true)}
                    >
                        {t("create_profile")}
                    </button>

                    <div className="border-t border-gray-700 pt-4">
                        <h3 className="text-sm font-semibold mb-2 text-gray-400">{t("game_version")}</h3>
                        <select
                            className="w-full p-2 bg-gray-900 border border-gray-700 rounded-sm text-gray-200 text-sm sm:text-base mb-2"
                            value={selectedVersion}
                            onChange={(e) => setSelectedVersion(e.target.value)}
                        >
                            {versions.map((version) => <option key={version} value={version}>{version}</option>)}
                        </select>
                        <button
                            className="az-btn az-hover-lift w-full p-2 az-bg-gray-900 border border-indigo-500 rounded-sm az-text-gray-200 hover:az-bg-gray-700 focus:outline-hidden text-sm sm:text-base"
                            onClick={() => setIsVersionSelectorOpen(true)}
                        >
                            {t('install_new_version', currentLanguage)}
                        </button>
                    </div>
                </div>

                <nav className="flex-1 py-4 mt-auto">
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
                </nav>

                <div className="p-4 sm:p-6 border-t border-gray-700">
                    <button
                        disabled={isDownloading || username === ""}
                        className="w-full py-2 sm:py-3 bg-green-600 hover:bg-green-700 text-white font-bold rounded-sm flex items-center justify-center disabled:bg-gray-500 text-sm sm:text-base"
                        onClick={handlePlay}
                    >
                        <Play size={18} className="mr-2"/>
                        {isDownloading ? t('downloading') : t("play")}
                    </button>
                    {isDownloading && (<div className="w-full bg-gray-700 rounded-full h-2 mt-4">
                        <div
                            className="bg-indigo-500 h-2 rounded-full"
                            style={{width: `${downloadProgress}%`}}
                        ></div>
                    </div>)}
                    <p className="text-xs mt-2 text-gray-400 text-center">{statusMessage}</p>
                </div>
            </div>

            <main className="flex-1 overflow-auto p-4 sm:p-6">
                {activeTab === 'home' && <HomeTab/>}
                {activeTab === 'settings' && <SettingsTab/>}
                {activeTab === 'mods' && <ModsTab/>}
            </main>
        </div>

        <LoginPopup isOpen={isLoginPopupOpen} onClose={() => {
            setIsLoginPopupPopupOpen(false);
            reloadProfiles();
        }}/>
        <VersionSelectorPopup
            isOpen={isVersionSelectorOpen}
            onClose={() => setIsVersionSelectorOpen(false)}
            onVersionSelect={(version) => invoke("download_version", {
                versionLoader: {
                    id: version.v,
                    date: version.d,
                    base: version.base
                }
            }).catch("Failed to download version")
                .then(() => {
                    window.location.reload();
                })}
            currentLanguage={currentLanguage}
        />
    </div>);
}

// Other components (NavItem, HomeTab, ModsTab, SettingsTab, etc.) remain the same
function NavItem({icon, title, active, onClick}) {
    const langClass = getCurrentLang() === "fa" ? 'font-vazir' : 'font-inter';
    return (<div
        className={`flex items-center px-6 py-3 cursor-pointer ${active ? 'bg-gray-700 border-r-4 border-indigo-500' : 'hover:bg-gray-700'}`}
        onClick={onClick}
    >
        <div className={`ml-3 ${active ? 'text-indigo-400' : 'text-gray-400'}`}>
            {icon}
        </div>
        <span className={`${active ? 'font-semibold' : ''} ${langClass}`}>{title}</span>
    </div>);
}

function HomeTab() {
    const newsArticles = [{
        title: 'مهم',
        content: 'در نسخه الفا ممکنه مشکلات زیادی ، کم و کسری زیادی باشه اگه چیزی به ذهنتون رسید و فیدبکی داشتید از گفتنش پرهیز نکنید @IntelligentFalcon',
        date: 'کمی پیش :)'
    }];

    return (<div className="p-8">
        <h2 className="text-2xl font-bold mb-6">{t("minecraft_news")}</h2>

        <div className="space-y-4">
            {newsArticles.map((article, index) => (<div key={index} className="bg-gray-800 p-6 rounded-sm">
                <h3 className="text-xl font-semibold mb-2">{article.title}</h3>
                <p className="text-gray-300 mb-3">{article.content}</p>
                <p className="text-sm text-indigo-400 italic">{article.date}</p>
            </div>))}
        </div>
    </div>);
}

function AddModPopup({isOpen, onClose}) {
    if (!isOpen) {
        return null;
    }

    const handleInstallMod = () => {
        invoke("install_mod_from_local").catch("Failed to install mod from local");
        console.log("Install mod clicked");
        onClose();
    };

    return (
        <div className="fixed inset-0 bg-black bg-opacity-70 flex justify-center items-center z-50">
            <div className="bg-gray-800 p-8 rounded-lg shadow-xl w-full max-w-sm relative text-gray-200">
                <button onClick={onClose}
                        className="absolute top-4 right-4 text-gray-400 hover:text-white transition-colors">
                    <X size={24}/>
                </button>
                <h2 className="text-3xl font-bold text-center mb-6">{t("install_mod")}</h2>
                <div className="space-y-4">
                    <button
                        onClick={handleInstallMod}
                        className="w-full p-3 bg-green-600 hover:bg-green-700 text-white font-bold rounded-sm transition-colors"
                    >
                        {t("select_mod_file")}
                    </button>
                    <button
                        onClick={() => console.log("Download from Modrinth clicked")}
                        className="w-full p-3 bg-blue-600 hover:bg-blue-700 text-white font-bold rounded-sm transition-colors"
                    >
                        Download from Modrinth
                    </button>
                    <button
                        onClick={() => console.log("Download from CurseForge clicked")}
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
    const [mods, setMods] = useState([]);
    const [isAddModPopupOpen, setAddModPopupOpen] = useState(false);
        useEffect(() => {
            if (mods.length < 1)

                invoke("get_mods").then((v) => {
                setMods(v);
            }).finally(() => {

                console.log("Loaded mods");
            });
        }, [mods])

    const handleToggleMod = (mod, enabled) => {
        // Here you would invoke a Tauri command to enable/disable the mod
        mod.enabled = enabled;
        console.log(`Toggling mod ${mod.name} to ${enabled}`);
        invoke("toggle_mod", {modInfo: mod, toggle: enabled}).catch("Error!");
        // For now, we'll just update the local state for visual feedback
        setMods(mods.map(m => m.name === mod.name ? mod : m));
    };
    
    const handleDeleteMod = (mod) => {
        console.log(`Deleting mod ${mod.name}`);
        // Here you would invoke a Tauri command to delete the mod
        // invoke("delete_mod", { modInfo: mod }).catch("Error!");
        // For now, we'll just update the local state for visual feedback
        setMods(mods.filter(m => m.name !== mod.name));
    };

    return (<div className="p-6">
        <div className="flex justify-between items-center mb-6">
            <h2 className="text-2xl font-bold">{t("mod_manager")}</h2>
            <div className="flex items-center">
                <input
                    type="text"
                    placeholder={t("mod_search")}
                    className="w-64 p-2 bg-gray-800 border border-gray-700 rounded-sm"
                />
                <button
                    onClick={() => setAddModPopupOpen(true)}
                    className="ml-2 p-2 bg-indigo-500 hover:bg-indigo-600 rounded-sm text-white"
                >
                    <Plus size={20}/>
                </button>
            </div>
        </div>

        <div className="space-y-3">
            {mods.map((mod, index) => (
                <div key={index} className="bg-gray-800 p-4 rounded-sm flex justify-between items-center">
                    <div>
                        <h3 className="font-semibold">{mod.name}</h3>
                        <p className="text-sm text-gray-400">{mod.description}</p>
                    </div>
                    <div className="flex items-center">
                        <span className="text-xs text-indigo-400 mr-3">{mod.version}</span>
                        <label htmlFor={`mod-toggle-${index}`} className="flex items-center cursor-pointer">
                            <div className="relative">
                                <input
                                    type="checkbox"
                                    id={`mod-toggle-${index}`}
                                    className="sr-only"
                                    checked={mod.enabled}
                                    onChange={(e) => handleToggleMod(mod, e.target.checked)}
                                />
                                <div className="block bg-gray-600 w-14 h-8 rounded-full"></div>
                                <div
                                    className="dot absolute left-1 top-1 bg-white w-6 h-6 rounded-full transition-transform"></div>
                            </div>
                        </label>
                        <button
                            onClick={() => handleDeleteMod(mod)}
                            className="ml-4 p-2 text-red-500 hover:text-red-400"
                        >
                            <Trash2 size={20}/>
                        </button>
                    </div>
                </div>))}
        </div>
        <AddModPopup isOpen={isAddModPopupOpen} onClose={() => {
            setMods([]);
            setAddModPopupOpen(false);
        }}/>
    </div>);
}

function RamUsageBar({totalRam, ramUsage, setRamUsage}) {
    const ramUsageGB = (ramUsage / 1024).toFixed(1);

    const handleRamChange = (event) => {
        const newRamValue = parseInt(event.target.value, 10);
        setRamUsage(newRamValue);
    };

    const handleRamChangeEnd = () => {
        invoke("set_ram_usage", {ramUsage: ramUsage}).catch(console.error);
    }

    return (<div className="bg-gray-800 p-6 rounded-sm">
        <h3 className="text-lg font-semibold mb-1">{t("mem_alloc_title")}</h3>
        <p className="text-sm text-gray-400 mb-4">{t("mem_alloc_description")}</p>
        <div className="flex items-center">
            <input type="range"
                   min="1.0"
                   max={totalRam > 1024 ? Math.round(totalRam * 10 / 1024) / 10 : 8}
                   step="0.5"
                   value={ramUsage}
                   onInput={handleRamChange}
                   onMouseUp={handleRamChangeEnd}
                   className="w-64"
            />
            <data id="ram_usage_label" className="ml-4 tabular-nums" value={ramUsageGB}>
                {ramUsageGB} GB
            </data>
        </div>
    </div>);
}

function SettingsTab() {
    const [totalRam, setTotalRam] = useState(0);
    const [ramUsage, setRamUsage] = useState(2048);

    useEffect(() => {
        invoke("get_total_ram").then(setTotalRam).catch(console.error);
        invoke("get_ram_usage").then(ram => {
            if (ram) setRamUsage(ram);
        }).catch(console.error);
    }, []);

    function saveSettings() {
        invoke("set_ram_usage", {ramUsage: ramUsage}).catch(console.error);
        invoke("save").catch(e => console.error("Couldn't save settings.", e));
        console.log("Settings Saved!");
    }

    return (<div className="p-6">
        <h2 className="text-2xl font-bold mb-6">{t("settings_tab")}</h2>

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
                <button className="px-4 py-2 bg-green-600 hover:bg-green-700 rounded-sm" onClick={saveSettings}>
                    {t("save_settings")}
                </button>
            </div>
        </div>
    </div>);
}