import {useEffect, useState} from 'react';
import {Settings, Package, Home, Newspaper, Play, X, Minus, ChevronRight} from 'lucide-react';
import {invoke} from "@tauri-apps/api/core";
import {listen} from '@tauri-apps/api/event';
import {LogicalSize, getCurrentWindow, currentMonitor} from '@tauri-apps/api/window';

/**
 * Important function, do not touch!
 */
async function lockWindow() {
    const monitor = await currentMonitor();

    // Something must be seriously wrong with the person using the app.
    if (!monitor) {
        console.log("What the actual fuck happened here?!");
    }

    // Calculate what size the window needs to be in order to keep the elements clean and
    // user-friendly according to the (poor ahh) user's aspect ratio
    const independantMultiplier = 1.2;
    const aspectRatio = monitor.size.width / monitor.size.height;
    const width = (monitor.size.width / aspectRatio) * independantMultiplier;
    const height = (monitor.size.height / aspectRatio) * independantMultiplier;

    const cwin = await getCurrentWindow();
    await cwin.setSize(new LogicalSize(width, height));
    await cwin.center();
    await cwin.setResizable(false);
    await cwin.setMaximizable(false);
    await cwin.setFocus();
}

/**
 * Important function! Do not move this lower, must be on top of everything else.
 */
lockWindow().catch(console.error);

// Have to put these here because SettingsTab and the main function both need it
var gRamUsage = 2048; // Default value
var gRamUsagePrettified = "2.0 GB"; // Default value

function setRamUsage(ramUsage) {
    gRamUsage = ramUsage;
    gRamUsagePrettified = (ramUsage / 1024).toFixed(1) + " GB";
    invoke("set_ram_usage", {ramUsage: ramUsage}).catch("").then();
    const text = document.getElementById("ram_usage_label");
    text.textContent = gRamUsagePrettified
}

export default function FalconClient() {
    const [activeTab, setActiveTab] = useState('home');
    const [downloadProgress, setDownloadProgress] = useState(0);
    const [isDownloading, setIsDownloading] = useState(false);
    const [versions, setVersions] = useState([]);
    const [selectedVersion, setSelectedVersion] = useState("");
    const [username, setUsername] = useState("");
    const [statusMessage, setStatusMessage] = useState('Ready to play');

    async function load_versions() {
        invoke("get_versions")
            .then((v) => setVersions(v))
            .catch((e) => console.error("Failed to fetch versions:", e));
    }

    if (versions.length === 0) load_versions().catch((e) => console.error("Not working!", e))

    invoke("get_ram_usage")
        .then(ramUsage => {
            setRamUsage(ramUsage);
        })
        .catch("Not working fuck");
    if (username === "")
        invoke("get_username").then((v) => setUsername(v)).catch("Couldn't get the username");


    useEffect(() => {
        async function registerEvents() {
            const unlisten = await listen('progress', (event) => {
                console.log('Progress:', event.payload);
                setStatusMessage(event.payload);
            });

            const unlistenbar = await listen('progressBar', (event) => {
                console.log('Progress:', event.payload);
                if (event.payload >= 100) {
                    setIsDownloading(false);
                }
                setDownloadProgress(event.payload);
            });
        }

        registerEvents().catch("Failed to register events");
    }, []);
    const handlePlay = async () => {
        if (selectedVersion === "") {
            setSelectedVersion(versions[0]);
        }
        setIsDownloading(true);
        invoke("set_username", {userName: username}).catch("Guess what? i couldn't save your username");
        invoke("save").catch("Saving configuration failed.");
        invoke("play_button_handler", {
            selectedVersion: selectedVersion
        }).catch((e) => console.error("Failed to launch game:", e));
        // Simulate download progress
        if (downloadProgress >= 100) {
            setIsDownloading(false);
            setDownloadProgress(0);
        }

    };

    return (<div className="flex flex-col w-full h-screen bg-gray-900 text-gray-200 overflow-hidden">
        {/* Header */}
        <div className="flex justify-between items-center px-6 py-3 bg-gray-800 border-b border-gray-700">
            <div className="flex items-center">
                <h1 className="text-xl font-bold text-indigo-400">Falcon Launcher</h1>
                <span className="ml-2 text-xs text-gray-400">v1.0.0</span>
            </div>
        </div>

        <div className="flex flex-1 overflow-hidden">
            {/* Sidebar */}
            <div className="w-64 bg-gray-800 flex flex-col">
                {/* User profile */}
                <div className="p-6 flex flex-col items-center">
                    <div
                        className="w-16 h-16 rounded-full bg-indigo-500 flex items-center justify-center text-white font-bold text-xl mb-4">
                        FC
                    </div>
                    <h2 className="text-lg font-semibold mb-4">Account Login</h2>
                    <input
                        type="text"
                        placeholder="Username/Email"
                        className="w-full mb-2 p-2 bg-gray-900 border border-indigo-500 rounded text-gray-200 focus:outline-none"
                        value={username}
                        onChange={(e) => setUsername(e.target.value)}

                        onInput={event => {
                            setUsername(event.target.value);
                        }}
                    />
                </div>

                {/* Version selection */}
                <div className="px-6 pb-4">
                    <label className="block text-sm font-semibold mb-2">Game Version</label>
                    <select className="w-full p-2 bg-gray-900 border border-gray-700 rounded text-gray-200"
                            onInput={event => {
                                setSelectedVersion(event.target.value)
                            }}>
                        {versions.map((version) => (<option key={version}>{version}</option>))}
                    </select>
                </div>

                {/* Navigation */}
                <div className="flex-1 py-4">
                    <NavItem
                        icon={<Home size={18}/>}
                        title="Home"
                        active={activeTab === 'home'}
                        onClick={() => setActiveTab('home')}
                    />
                    <NavItem
                        icon={<Package size={18}/>}
                        title="Mods"
                        active={activeTab === 'mods'}
                        onClick={() => setActiveTab('mods')}
                    />
                    <NavItem
                        icon={<Settings size={18}/>}
                        title="Settings"
                        active={activeTab === 'settings'}
                        onClick={() => setActiveTab('settings')}
                    />
                    <NavItem
                        icon={<Newspaper size={18}/>}
                        title="News"
                        active={activeTab === 'news'}
                        onClick={() => setActiveTab('news')}
                    />
                </div>

                {/* Play button and status */}
                <div className="p-6 border-t border-gray-700">
                    <button disabled={isDownloading}
                            className="w-full py-3 bg-green-600 hover:bg-green-700 text-white font-bold rounded flex items-center justify-center"
                            onClick={handlePlay}
                    >
                        <Play size={18} className="mr-2"/>
                        PLAY
                    </button>
                    {isDownloading && (<div className="w-full bg-gray-700 rounded-full h-2 mt-4">
                        <div
                            className="bg-indigo-500 h-2 rounded-full"
                            style={{width: `${downloadProgress}%`}}
                        ></div>
                    </div>)}
                    <p className="text-xs mt-2 text-gray-400">{statusMessage}</p>
                </div>
            </div>

            {/* Main content */}
            <div className="flex-1 overflow-auto">
                {activeTab === 'home' && <HomeTab/>}
                {activeTab === 'mods' && <ModsTab/>}
                {activeTab === 'settings' && <SettingsTab/>}
                {activeTab === 'news' && <NewsTab/>}
            </div>
        </div>
    </div>);
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
    return (<div className="p-8">
        <div className="text-center mb-12">
            <h1 className="text-4xl font-bold mb-2">Welcome to FalconLauncher</h1>
            <p className="text-xl text-indigo-400">Open source and Free Minecraft Launcher :)</p>
        </div>

        <div className="grid grid-cols-2 gap-6">
            {[{title: 'Performance', description: 'Optimized for speed and smooth gameplay'}, {
                title: 'Mods', description: 'Easy installation and management of mods'
            }, {title: 'Customization', description: 'Personalize your Minecraft experience'}, {
                title: 'Security', description: 'Safe and secure gaming environment'
            }].map((feature, index) => (<div key={index} className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-xl font-bold text-indigo-400 mb-2">{feature.title}</h3>
                <p className="text-gray-300">{feature.description}</p>
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
            <h2 className="text-2xl font-bold">Mod Manager</h2>
            <input
                type="text"
                placeholder="Search mods..."
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


function SettingsTab() {
    function save() {
        invoke("set_ram_usage", {ramUsage: gRamUsage}).catch("Aw man you screwed it up");
        invoke("save").catch("Couldn't save file.")
    }

    return (<div className="p-6">
        <h2 className="text-2xl font-bold mb-6">Settings</h2>

        <div className="space-y-6">
            {/* Memory Settings */}
            <div className="bg-gray-800 p-6 rounded">
                <h3 className="text-lg font-semibold mb-1">Memory Allocation</h3>
                <p className="text-sm text-gray-400 mb-4">Adjust how much RAM is allocated to Minecraft</p>

                <div className="flex items-center">
                    <input type="range" min="1024" max="32768" defaultValue={gRamUsage} onInput={event => {
                        setRamUsage(event.target.valueAsNumber);

                        /*
                            WARNING: I'm commenting this because this is so shit.
                            WARNING: For the love of god change this to not write in a file every single time that slider is changed
                            This will destroy the user's CPU and is terrible for efficiency.
                            NOTE: Remove this comment when fixed;
                        */


                    }} className="w-64"/>

                    {/* NOTE: Fix this not updating itself */}
                    <data id="ram_usage_label" className="ml-4" value={gRamUsagePrettified}>{gRamUsagePrettified}</data>
                </div>
            </div>

            {/* Java Settings */}
            <div className="bg-gray-800 p-6 rounded">
                <h3 className="text-lg font-semibold mb-1">Java Settings</h3>
                <p className="text-sm text-gray-400 mb-4">Select which Java version to use</p>

                <select className="w-64 p-2 bg-gray-700 border border-gray-600 rounded">
                    <option>Auto-detect</option>
                    <option>Java 8</option>
                    <option>Java 11</option>
                    <option>Java 17</option>
                </select>
            </div>

            {/* Launch Options */}
            <div className="bg-gray-800 p-6 rounded">
                <h3 className="text-lg font-semibold mb-1">Launch Options</h3>
                <p className="text-sm text-gray-400 mb-4">Configure how Minecraft starts</p>

                <div className="space-y-3">
                    <div className="flex items-center">
                        <input type="checkbox" id="fullscreen" className="mr-2"/>
                        <label htmlFor="fullscreen">Launch in fullscreen</label>
                    </div>
                    <div className="flex items-center">
                        <input type="checkbox" id="close-launcher" className="mr-2"/>
                        <label htmlFor="close-launcher">Close launcher when game starts</label>
                    </div>
                    <div className="flex items-center">
                        <input type="checkbox" id="check-updates" className="mr-2"/>
                        <label htmlFor="check-updates">Check for updates on startup</label>
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

function NewsTab() {
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

    return (<div className="p-6">
        <h2 className="text-2xl font-bold mb-6">Minecraft News</h2>

        <div className="space-y-4">
            {newsArticles.map((article, index) => (<div key={index} className="bg-gray-800 p-6 rounded">
                <h3 className="text-xl font-semibold mb-2">{article.title}</h3>
                <p className="text-gray-300 mb-3">{article.content}</p>
                <p className="text-sm text-indigo-400 italic">{article.date}</p>
            </div>))}
        </div>
    </div>);
}

