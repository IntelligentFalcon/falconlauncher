import { useState, useEffect } from "react";
import {
    Search, X, ChevronLeft, Download, CheckSquare, Square,
    Package, AlertCircle, Code, Book, MessageSquare, Bug, Heart, Layers, Loader2
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { listen } from "@tauri-apps/api/event";
import { useModsState } from "@/context/mods-context";
import { LoadingSwap } from "@/components/ui/animated/swapper";
import { useBackendMutation } from "@/hooks/use-backend";

import {
    Combobox,
    ComboboxContent,
    ComboboxEmpty,
    ComboboxInput,
    ComboboxItem,
    ComboboxList,
} from "@/components/ui/combobox";

import type {
    ModrinthSearchResultMod,
    ModrinthSearchResult,
    ModrinthVersion,
    ModrinthMod,
    DependencyTuple
} from "@/invokes";

export interface VersionNameBase {
    name: string;
    base: string;
    loader: string; // <-- Added loader property
}

export interface ModDownloadProgress {
    stage?: string;
    stage_name?: string;
    current_file?: number;
    total_files?: number;
    current_bytes?: number;
    total_bytes?: number;
    file_name?: string;
    global_percentage?: number;
    percentage?: number;
}

interface DependencyState {
    version: ModrinthVersion;
    type: string;
    selected: boolean;
}

const SORT_OPTIONS = ["Relevance", "Downloads", "Follows", "Newest", "Updated"];

const parseMarkdownToHtml = (text: string) => {
    if (!text) return "";
    let html = text;

    html = html.replace(/!\[([^\]]*)]\(([^)]+)\)/g, '<img src="$2" alt="$1" />');
    html = html.replace(/\[([^\]]+)]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noreferrer">$1</a>');
    html = html.replace(/^### (.*$)/gim, '<h3>$1</h3>');
    html = html.replace(/^## (.*$)/gim, '<h2>$1</h2>');
    html = html.replace(/^# (.*$)/gim, '<h1>$1</h1>');
    html = html.replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>');
    html = html.replace(/\*([^*]+)\*/g, '<em>$1</em>');
    html = html.replace(/```([\s\S]*?)```/g, '<pre><code>$1</code></pre>');
    html = html.replace(/`([^`]+)`/g, '<code>$1</code>');

    return html;
};

export function ModrinthDownloadModal({
                                          isOpen,
                                          onClose,
                                      }: {
    isOpen: boolean;
    onClose: () => void;
}) {
    const { t } = useTranslation();
    const { selectedVersion, installedVersions } = useModsState();

    // Find the base vanilla Minecraft version & loader for Modrinth searching & compatibility
    const currentVersionObj = installedVersions?.find(
        (v: VersionNameBase ) =>
            v === selectedVersion
    );
    const baseVersion =
        (currentVersionObj?.base)
            ? currentVersionObj.base
            : currentVersionObj?.name;

    const currentLoader = currentVersionObj?.loader || "vanilla"; // <-- Extract loader from the updated rust struct

    const [provider, setProvider] = useState<"modrinth" | "curseforge">("modrinth");
    const [step, setStep] = useState<"search" | "details" | "dependencies">("search");
    const [query, setQuery] = useState("");
    const [searchIndex, setSearchIndex] = useState("Relevance");

    const [searchResults, setSearchResults] = useState<ModrinthSearchResultMod[]>([]);
    const [selectedMod, setSelectedMod] = useState<ModrinthSearchResultMod | null>(null);
    const [fullProject, setFullProject] = useState<ModrinthMod | null>(null);
    const [versions, setVersions] = useState<ModrinthVersion[]>([]);
    const [selectedModVersion, setSelectedModVersion] = useState<ModrinthVersion | null>(null);
    const [dependencies, setDependencies] = useState<DependencyState[]>([]);

    // Download & Progress Tracker State
    const [downloadProgress, setDownloadProgress] = useState<ModDownloadProgress | null>(null);
    const [isDownloadingState, setIsDownloadingState] = useState(false);

    const [fullscreenImage, setFullscreenImage] = useState<string | undefined>();

    const { mutateAsync: searchProjects, isPending: isSearching } = useBackendMutation({ name: "search_for_modrinth_project" });
    const { mutateAsync: fetchVersions, isPending: isLoadingVersions } = useBackendMutation({ name: "list_modrinth_mod_versions" });
    const { mutateAsync: fetchProject, isPending: isLoadingProject } = useBackendMutation({ name: "get_modrinth_projects" });
    const { mutateAsync: fetchDependencies, isPending: isLoadingDependencies } = useBackendMutation({ name: "get_modrinth_mod_dependencies" });
    const { mutateAsync: downloadVersion, isPending: isDownloadingMutation } = useBackendMutation({ name: "download_modrinth_mod_version" });

    // Listen to "progress-bar" event from Tauri backend
    useEffect(() => {
        if (!isOpen) return;

        const unlistenPromise = listen<ModDownloadProgress>("progress-bar", (event) => {
            setDownloadProgress(event.payload);
        });

        return () => {
            unlistenPromise.then((unlisten) => unlisten());
        };
    }, [isOpen]);

    const executeSearch = async (currentQuery: string, currentIndexDisplay: string) => {
        if (provider !== "modrinth" || !baseVersion) return;

        // Modrinth API facets filter by vanilla base version (e.g., 1.20.1) and loader
        const facetList = [
            [`versions:${baseVersion}`],
            ["project_type:mod"],
        ];

        // Add loader facet if it's not vanilla
        if (currentLoader !== "vanilla") {
            facetList.push([`categories:${currentLoader}`]);
        }

        const facets = JSON.stringify(facetList);
        const apiIndex = currentIndexDisplay.toLowerCase();

        try {
            const res = await searchProjects({
                name: currentQuery,
                facets,
                index: apiIndex,
                offset: 0,
                limit: 20,
            }) as ModrinthSearchResult;

            setSearchResults(res.hits);
        } catch (err) {
            console.error("Search failed:", err);
        }
    };

    const handleSearch = (e: React.FormEvent) => {
        e.preventDefault();
        executeSearch(query, searchIndex);
    };

    useEffect(() => {
        if (isOpen && step === "search" && baseVersion && provider === "modrinth") {
            executeSearch(query, searchIndex);
        }
    }, [isOpen, searchIndex, baseVersion, provider]);

    const handleProviderSwitch = (newProvider: "modrinth" | "curseforge") => {
        if (newProvider === provider || isDownloadingState) return;
        setProvider(newProvider);
        setStep("search");
        setQuery("");
        setSearchResults([]);
        setSelectedMod(null);
        setFullProject(null);
    };

    const handleSelectMod = async (mod: ModrinthSearchResultMod) => {
        setSelectedMod(mod);
        setStep("details");
        setFullProject(null);

        try {
            const [versionsRes, projectRes] = await Promise.all([
                fetchVersions({ projectId: mod.project_id }) as Promise<ModrinthVersion[]>,
                fetchProject({ projectId: mod.project_id }) as Promise<ModrinthMod>
            ]);

            // Filter compatible releases using base Minecraft version & loader
            const compatibleVersions = versionsRes.filter((v) => {
                const matchesVersion = v.game_versions?.includes(baseVersion ?? "");
                const matchesLoader = currentLoader === "vanilla" || v.loaders?.includes(currentLoader);
                return matchesVersion && matchesLoader;
            });

            setVersions(compatibleVersions);
            setFullProject(projectRes);
        } catch (err) {
            console.error("Failed to fetch mod details or versions:", err);
        }
    };

    const handleSelectVersion = async (version: ModrinthVersion) => {
        setSelectedModVersion(version);
        setStep("dependencies");

        try {
            const depsRes = await fetchDependencies({ version }) as DependencyTuple[];
            const parsedDeps: DependencyState[] = depsRes.map(([depVersion, type]) => ({
                version: depVersion,
                type,
                selected: type === "required",
            }));
            setDependencies(parsedDeps);
        } catch (err) {
            console.error("Failed to fetch dependencies:", err);
        }
    };

    const toggleDependency = (id: string) => {
        if (isDownloadingState) return;
        setDependencies((prev) =>
            prev.map((d) => (d.version.id === id ? { ...d, selected: !d.selected } : d))
        );
    };

    const handleDownload = async () => {
        if (!selectedModVersion || !selectedVersion || isDownloadingState) return;

        setIsDownloadingState(true);
        setDownloadProgress(null);

        try {
            // Save mod to the profile/instance name on disk
            await downloadVersion({ version: selectedModVersion, name: selectedVersion.name });

            // Download selected dependencies
            const selectedDeps = dependencies.filter((d) => d.selected);
            for (const dep of selectedDeps) {
                await downloadVersion({ version: dep.version, name: selectedVersion.name });
            }

            handleClose();
        } catch (err) {
            console.error("Failed to download mod or dependencies:", err);
        } finally {
            setIsDownloadingState(false);
            setDownloadProgress(null);
        }
    };

    const handleClose = () => {
        if (isDownloadingState) return;
        setStep("search");
        setQuery("");
        setProvider("modrinth");
        setDownloadProgress(null);
        onClose();
    };

    if (!isOpen) return null;

    const isDownloading = isDownloadingState || isDownloadingMutation;
    const progressPercent = Math.min(
        Math.max(
            Number(downloadProgress?.global_percentage ?? downloadProgress?.percentage ?? 0),
            0
        ),
        100
    );

    return (
        <>
            <div className="fixed inset-0 z-50 flex items-center justify-center bg-background/80 p-4 backdrop-blur-sm">
                <div className="relative flex h-full max-h-[700px] w-full max-w-5xl overflow-hidden rounded-2xl border border-border/40 bg-card shadow-lg">

                    {/* PROGRESS TRACKER POPUP OVERLAY */}
                    {isDownloading && (
                        <div className="absolute inset-0 z-40 flex flex-col items-center justify-center bg-background/90 p-6 backdrop-blur-md animate-in fade-in duration-200">
                            <div className="flex w-full max-w-md flex-col items-center gap-5 rounded-2xl border border-border/60 bg-card/90 p-6 shadow-2xl">
                                <div className="rounded-full border border-primary/30 bg-primary/10 p-3.5 text-primary">
                                    <Loader2 size={32} className="animate-spin" />
                                </div>

                                <div className="w-full text-center">
                                    <h3 className="font-bold text-lg text-foreground">
                                        {t("modrinthModal.downloadingTitle", "Downloading Mod")}
                                    </h3>
                                    <p className="mt-1 text-xs text-muted-foreground truncate">
                                        {downloadProgress?.stage_name ||
                                            downloadProgress?.file_name ||
                                            selectedModVersion?.name ||
                                            t("modrinthModal.fetchingFiles", "Fetching files...")}
                                    </p>
                                </div>

                                {/* Progress Bar Container */}
                                <div className="w-full space-y-2">
                                    <div className="h-2.5 w-full overflow-hidden rounded-full border border-border/40 bg-secondary/50 p-0.5 shadow-inner">
                                        <div
                                            className="h-full rounded-full bg-primary transition-all duration-200 ease-out"
                                            style={{ width: `${progressPercent}%` }}
                                        />
                                    </div>
                                    <div className="flex items-center justify-between text-xs text-muted-foreground">
                                        <span className="truncate max-w-[70%] font-mono text-[11px]">
                                            {downloadProgress?.file_name ?? ""}
                                        </span>
                                        <span className="font-bold">{progressPercent.toFixed(0)}%</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    )}

                    {/* SIDEBAR */}
                    <div className="w-56 shrink-0 border-r border-border/40 bg-secondary/10 p-4 flex flex-col gap-2">
                        <h3 className="mb-2 px-2 text-xs font-bold uppercase tracking-wider text-muted-foreground">
                            {t("modrinthModal.sources")}
                        </h3>

                        <button
                            onClick={() => handleProviderSwitch("modrinth")}
                            disabled={isDownloading}
                            className={`flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-sm font-medium transition-all ${
                                provider === "modrinth"
                                    ? "bg-primary text-primary-foreground shadow-sm"
                                    : "text-muted-foreground hover:bg-secondary/60 hover:text-foreground"
                            }`}
                        >
                            <Package size={18} className={provider === "modrinth" ? "text-primary-foreground" : "text-muted-foreground"} />
                            Modrinth
                        </button>
                    </div>

                    {/* MAIN CONTENT AREA */}
                    <div className="flex flex-1 flex-col overflow-hidden bg-background">
                        {/* HEADER */}
                        <div className="flex items-center justify-between border-b border-border/40 bg-secondary/20 p-4">
                            <div className="flex items-center gap-3">
                                {step !== "search" && (
                                    <button
                                        disabled={isDownloading}
                                        onClick={() => setStep(step === "dependencies" ? "details" : "search")}
                                        className="rounded-lg p-1.5 hover:bg-secondary transition-colors disabled:opacity-50"
                                    >
                                        <ChevronLeft size={18} />
                                    </button>
                                )}
                                <h2 className="font-semibold text-foreground">
                                    {step === "search" && t("modrinthModal.searchTitle", { provider: provider === "modrinth" ? "Modrinth" : "CurseForge" })}
                                    {step === "details" && selectedMod?.title}
                                    {step === "dependencies" && t("modrinthModal.reviewDependencies")}
                                </h2>
                            </div>
                            <button
                                disabled={isDownloading}
                                onClick={handleClose}
                                className="rounded-lg p-1.5 hover:bg-secondary transition-colors disabled:opacity-50"
                            >
                                <X size={18} />
                            </button>
                        </div>

                        {/* BODY */}
                        <div className="flex-1 overflow-y-auto p-5">

                            {/* CURSEFORGE PLACEHOLDER */}
                            {provider === "curseforge" && (
                                <div className="flex h-full flex-col items-center justify-center text-center text-muted-foreground">
                                    <div className="mb-6 flex h-20 w-20 items-center justify-center rounded-2xl bg-secondary/50">
                                        <Layers size={40} className="opacity-50" />
                                    </div>
                                    <h3 className="mb-2 text-xl font-bold text-foreground">{t("modrinthModal.curseforgeTitle")}</h3>
                                    <p className="max-w-xs text-sm">{t("modrinthModal.curseforgeDesc")}</p>
                                </div>
                            )}

                            {/* MODRINTH SEARCH STEP */}
                            {provider === "modrinth" && step === "search" && (
                                <div className="flex flex-col gap-5">
                                    <div className="flex flex-wrap items-center gap-3">
                                        <form onSubmit={handleSearch} className="relative flex-1 min-w-[200px]">
                                            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" size={16} />
                                            <input
                                                autoFocus
                                                className="h-11 w-full rounded-xl border border-border/50 bg-secondary/10 py-2.5 pl-9 pr-4 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50"
                                                placeholder={t("modrinthModal.searchPlaceholder", { version: baseVersion })}
                                                value={query}
                                                onChange={(e) => setQuery(e.target.value)}
                                            />
                                        </form>

                                        <div className="w-48 shrink-0">
                                            <Combobox
                                                autoHighlight
                                                items={SORT_OPTIONS}
                                                onValueChange={(val) => setSearchIndex(val ?? "Relevance")}
                                                value={searchIndex}
                                            >
                                                <ComboboxInput
                                                    className="h-11 w-full rounded-xl border border-border/50 bg-secondary/10 px-3 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 cursor-pointer"
                                                    placeholder={t("modrinthModal.sortBy")}
                                                    value={t(`modrinthModal.sort.${searchIndex.toLowerCase()}`)}
                                                    readOnly
                                                />
                                                <ComboboxContent className="border-border/50 bg-card">
                                                    <ComboboxEmpty>{t("modrinthModal.noOptions")}</ComboboxEmpty>
                                                    <ComboboxList>
                                                        {(item) => (
                                                            <ComboboxItem
                                                                key={item}
                                                                value={item}
                                                                className="cursor-pointer hover:bg-secondary"
                                                            >
                                                                {t(`modrinthModal.sort.${item.toLowerCase()}`)}
                                                            </ComboboxItem>
                                                        )}
                                                    </ComboboxList>
                                                </ComboboxContent>
                                            </Combobox>
                                        </div>
                                    </div>

                                    <LoadingSwap isLoading={isSearching}>
                                        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                                            {searchResults.map((mod) => (
                                                <div
                                                    key={mod.project_id}
                                                    onClick={() => handleSelectMod(mod)}
                                                    className="flex cursor-pointer gap-4 rounded-xl border border-border/40 bg-secondary/10 p-3.5 transition-all hover:bg-secondary/40 hover:shadow-sm"
                                                >
                                                    {mod.icon_url ? (
                                                        <img src={mod.icon_url} alt={mod.title} className="h-14 w-14 rounded-xl bg-background object-cover shrink-0 shadow-sm" />
                                                    ) : (
                                                        <div className="flex h-14 w-14 items-center justify-center rounded-xl bg-background/50 shrink-0 shadow-sm border border-border/20">
                                                            <Package size={24} className="text-muted-foreground" />
                                                        </div>
                                                    )}
                                                    <div className="flex flex-col flex-1 justify-center overflow-hidden">
                                                        <div className="flex mb-1">
                                                            <span className="font-bold text-base leading-tight text-foreground truncate">{mod.title}</span>
                                                        </div>
                                                        <span className="text-xs text-muted-foreground line-clamp-2 leading-relaxed">{mod.description}</span>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                        {searchResults.length === 0 && !isSearching && (
                                            <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
                                                <Package size={56} className="mb-4 opacity-20" />
                                                <p className="text-sm font-medium text-foreground">{t("modrinthModal.noModsFound")}</p>
                                                <p className="text-xs mt-1">{t("modrinthModal.adjustQuery")}</p>
                                            </div>
                                        )}
                                    </LoadingSwap>
                                </div>
                            )}

                            {/* MODRINTH DETAILS STEP */}
                            {provider === "modrinth" && step === "details" && selectedMod && (
                                <div className="flex flex-col gap-6 select-text">

                                    <div className="flex items-start gap-5">
                                        {selectedMod.icon_url ? (
                                            <img src={selectedMod.icon_url} alt={selectedMod.title} className="h-24 w-24 shrink-0 rounded-2xl shadow-sm object-cover bg-secondary border border-border/30" />
                                        ) : (
                                            <div className="flex h-24 w-24 shrink-0 items-center justify-center rounded-2xl bg-secondary/50 border border-border/30">
                                                <Package size={40} className="text-muted-foreground" />
                                            </div>
                                        )}
                                        <div className="flex flex-col pt-1">
                                            <h1 className="text-2xl font-bold leading-tight">{selectedMod.title}</h1>
                                            <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-sm font-medium text-muted-foreground mt-1.5">
                                                <span>{t("modrinthModal.by")} <span className="text-foreground">{selectedMod.author}</span></span>
                                                <span>•</span>
                                                <span className="uppercase text-foreground">
                                                    {fullProject?.license?.name || selectedMod.license}
                                                </span>
                                                <span>•</span>
                                                <span>{t("modrinthModal.downloads", { count: selectedMod.downloads.toLocaleString() })}</span>
                                            </div>
                                            <p className="text-sm text-muted-foreground mt-3 leading-relaxed max-w-3xl">{selectedMod.description}</p>
                                        </div>
                                    </div>

                                    <LoadingSwap isLoading={isLoadingProject}>
                                        {fullProject && (
                                            <div className="flex flex-col gap-8">
                                                <div className="flex flex-wrap gap-2.5 select-none">
                                                    {fullProject.source_url && (
                                                        <a href={fullProject.source_url} target="_blank" rel="noreferrer" className="flex items-center gap-1.5 rounded-lg border border-border/50 bg-secondary/30 px-3.5 py-1.5 text-xs font-medium text-foreground hover:bg-secondary transition-colors">
                                                            <Code size={14} className="text-muted-foreground" /> {t("modrinthModal.links.source")}
                                                        </a>
                                                    )}
                                                    {fullProject.wiki_url && (
                                                        <a href={fullProject.wiki_url} target="_blank" rel="noreferrer" className="flex items-center gap-1.5 rounded-lg border border-border/50 bg-secondary/30 px-3.5 py-1.5 text-xs font-medium text-foreground hover:bg-secondary transition-colors">
                                                            <Book size={14} className="text-muted-foreground" /> {t("modrinthModal.links.wiki")}
                                                        </a>
                                                    )}
                                                    {fullProject.discord_url && (
                                                        <a href={fullProject.discord_url} target="_blank" rel="noreferrer" className="flex items-center gap-1.5 rounded-lg border border-border/50 bg-secondary/30 px-3.5 py-1.5 text-xs font-medium text-foreground hover:bg-secondary transition-colors">
                                                            <MessageSquare size={14} className="text-muted-foreground" /> {t("modrinthModal.links.discord")}
                                                        </a>
                                                    )}
                                                    {fullProject.issues_url && (
                                                        <a href={fullProject.issues_url} target="_blank" rel="noreferrer" className="flex items-center gap-1.5 rounded-lg border border-border/50 bg-secondary/30 px-3.5 py-1.5 text-xs font-medium text-foreground hover:bg-secondary transition-colors">
                                                            <Bug size={14} className="text-muted-foreground" /> {t("modrinthModal.links.issues")}
                                                        </a>
                                                    )}
                                                    {fullProject.donation_urls && fullProject.donation_urls.map(donation => (
                                                        <a key={donation.id} href={donation.url ? donation.url : undefined} target="_blank" rel="noreferrer" className="flex items-center gap-1.5 rounded-lg border border-border/50 bg-primary/10 px-3.5 py-1.5 text-xs font-medium text-primary hover:bg-primary/20 transition-colors">
                                                            <Heart size={14} /> {donation.platform}
                                                        </a>
                                                    ))}
                                                </div>

                                                {fullProject.gallery && fullProject.gallery.length > 0 && (
                                                    <div className="select-none">
                                                        <h3 className="mb-3 font-semibold text-sm">{t("modrinthModal.gallery")}</h3>
                                                        <div className="flex gap-4 overflow-x-auto pb-4 snap-x hide-scrollbar">
                                                            {fullProject.gallery.map((img, i) => (
                                                                <div
                                                                    key={i}
                                                                    className="shrink-0 snap-center cursor-pointer"
                                                                    onClick={() => setFullscreenImage(img.url)}
                                                                >
                                                                    <img
                                                                        src={img.url}
                                                                        alt={img.title || `${fullProject.title} gallery image ${i + 1}`}
                                                                        className="h-40 w-auto rounded-xl border border-border/30 object-cover shadow-sm transition-transform hover:scale-[1.02]"
                                                                        draggable={false}
                                                                    />
                                                                </div>
                                                            ))}
                                                        </div>
                                                    </div>
                                                )}

                                                {fullProject.body && (
                                                    <div className="flex flex-col gap-3">
                                                        <h3 className="font-semibold text-sm">{t("modrinthModal.aboutMod")}</h3>
                                                        <div
                                                            className="overflow-hidden rounded-xl border border-border/40 bg-secondary/5 p-5 text-sm text-foreground/90 leading-relaxed shadow-sm [&_a]:text-primary [&_a]:underline [&_img]:max-w-full [&_img]:h-auto [&_img]:rounded-xl [&_img]:my-4 [&_img]:border [&_img]:border-border/30 [&_h1]:text-2xl [&_h1]:font-bold [&_h1]:mt-6 [&_h1]:mb-4 [&_h2]:text-xl [&_h2]:font-bold [&_h2]:mt-5 [&_h2]:mb-3 [&_h3]:font-semibold [&_h3]:mt-4 [&_h3]:mb-2 [&_ul]:list-disc [&_ul]:ml-6 [&_ul]:my-3 [&_ol]:list-decimal [&_ol]:ml-6 [&_ol]:my-3 [&_p]:mb-4 [&_pre]:bg-background [&_pre]:p-4 [&_pre]:rounded-xl [&_pre]:border [&_pre]:border-border/50 [&_pre]:overflow-x-auto [&_pre]:my-4 [&_code]:bg-secondary/50 [&_code]:px-1.5 [&_code]:py-0.5 [&_code]:rounded-md [&_code]:font-mono [&_code]:text-[13px]"
                                                            dangerouslySetInnerHTML={{ __html: parseMarkdownToHtml(fullProject.body) }}
                                                        />
                                                    </div>
                                                )}
                                            </div>
                                        )}
                                    </LoadingSwap>

                                    <div className="select-none mt-4">
                                        <h3 className="mb-3 font-semibold text-sm">{t("modrinthModal.availableVersions", { version: baseVersion })}</h3>
                                        <LoadingSwap isLoading={isLoadingVersions}>
                                            <div className="flex flex-col gap-2">
                                                {versions.map((ver) => (
                                                    <div key={ver.id} className="flex items-center justify-between rounded-xl border border-border/40 bg-secondary/10 p-3 hover:bg-secondary/20 transition-colors">
                                                        <div className="flex flex-col">
                                                            <span className="font-medium text-sm">{ver.name || ver.version_number}</span>
                                                            <span className="text-xs text-muted-foreground mt-0.5">
                                                                {new Date(ver.date_published).toLocaleDateString()} • {ver.version_type} • {ver.loaders.join(", ")}
                                                            </span>
                                                        </div>
                                                        <button
                                                            onClick={() => handleSelectVersion(ver)}
                                                            className="rounded-lg bg-primary/10 px-4 py-2 text-xs font-semibold text-primary hover:bg-primary/20 transition-colors"
                                                        >
                                                            {t("modrinthModal.select")}
                                                        </button>
                                                    </div>
                                                ))}
                                                {versions.length === 0 && (
                                                    <div className="flex items-center gap-2 text-destructive bg-destructive/10 p-4 rounded-xl text-sm">
                                                        <AlertCircle size={16} />
                                                        {t("modrinthModal.noVersionsFound", { version: baseVersion })}
                                                    </div>
                                                )}
                                            </div>
                                        </LoadingSwap>
                                    </div>
                                </div>
                            )}

                            {/* DEPENDENCIES STEP */}
                            {provider === "modrinth" && step === "dependencies" && (
                                <div className="flex h-full flex-col">
                                    <div className="mb-4 text-sm text-muted-foreground">
                                        {t("modrinthModal.dependenciesIntro")}
                                    </div>

                                    <LoadingSwap isLoading={isLoadingDependencies}>
                                        <div className="flex-1 flex flex-col gap-2 overflow-y-auto">
                                            {dependencies.length === 0 ? (
                                                <div className="flex h-32 items-center justify-center rounded-xl border border-dashed border-border/50 bg-secondary/5 text-sm text-muted-foreground">
                                                    {t("modrinthModal.noDependencies")}
                                                </div>
                                            ) : (
                                                dependencies.map((dep) => (
                                                    <button
                                                        key={dep.version.id}
                                                        onClick={() => toggleDependency(dep.version.id)}
                                                        className={`flex items-center gap-4 rounded-xl border p-4 text-left transition-colors ${
                                                            dep.selected ? "border-primary bg-primary/5" : "border-border/40 bg-secondary/10 hover:bg-secondary/20"
                                                        }`}
                                                    >
                                                        <div className={dep.selected ? "text-primary" : "text-muted-foreground"}>
                                                            {dep.selected ? <CheckSquare size={20} /> : <Square size={20} />}
                                                        </div>
                                                        <div className="flex flex-col">
                                                            <span className="font-medium text-sm">{dep.version.name || dep.version.version_number}</span>
                                                            <span className="text-xs text-muted-foreground capitalize mt-1">
                                                                {t("modrinthModal.dependencyType", { type: dep.type })}
                                                            </span>
                                                        </div>
                                                    </button>
                                                ))
                                            )}
                                        </div>
                                    </LoadingSwap>
                                </div>
                            )}
                        </div>

                        {/* FOOTER ACTIONS */}
                        {step === "dependencies" && (
                            <div className="border-t border-border/40 bg-secondary/20 p-4">
                                <button
                                    onClick={handleDownload}
                                    disabled={isDownloading || isLoadingDependencies}
                                    className={`flex w-full items-center justify-center gap-2 rounded-xl py-3 font-semibold shadow-sm transition-colors ${
                                        isDownloading || isLoadingDependencies
                                            ? "cursor-not-allowed bg-secondary text-muted-foreground"
                                            : "bg-primary text-primary-foreground hover:bg-primary/90"
                                    }`}
                                >
                                    <Download size={18} className={isDownloading ? "animate-bounce" : ""} />
                                    {isDownloading ? t("modrinthModal.downloading") : t("modrinthModal.downloadSelected")}
                                </button>
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* FULLSCREEN IMAGE OVERLAY */}
            {fullscreenImage && (
                <div
                    className="fixed inset-0 z-[60] flex items-center justify-center bg-black/95 p-4 backdrop-blur-sm transition-opacity cursor-zoom-out"
                    onClick={() => setFullscreenImage(undefined)}
                >
                    <img
                        src={fullscreenImage}
                        alt={t("modrinthModal.fullscreenAlt")}
                        className="max-h-full max-w-full object-contain rounded-xl shadow-2xl"
                        draggable={false}
                    />
                    <button
                        className="absolute top-6 right-6 rounded-full bg-white/10 p-2.5 text-white hover:bg-white hover:text-black transition-colors backdrop-blur-md"
                        onClick={(e) => {
                            e.stopPropagation();
                            setFullscreenImage(undefined);
                        }}
                    >
                        <X size={24} />
                    </button>
                </div>
            )}
        </>
    );
}