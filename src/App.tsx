import { listen } from "@tauri-apps/api/event";
import { Blocks, FolderOpen, HardDriveDownload, Puzzle, SlidersHorizontal, type LucideIcon } from "lucide-react";
import { useEffect } from "react";
import { Toasts } from "./components/Toasts";
import { BinariesPage } from "./features/BinariesPage";
import { EnvPage } from "./features/EnvPage";
import { ExtensionsPage } from "./features/ExtensionsPage";
import { ServicesPage } from "./features/ServicesPage";
import { api } from "./lib/api";
import { useOverview } from "./lib/queries";
import { useNav, useProgress, type PageId, type Progress } from "./lib/stores";

const PAGES: Record<PageId, { label: string; subtitle: string; icon: LucideIcon; view: () => React.JSX.Element }> = {
  services: { label: "Inicio", subtitle: "Enciende y apaga tus servidores con un clic.", icon: Blocks, view: ServicesPage },
  binaries: { label: "Programas", subtitle: "Descarga PHP, Apache, MySQL y más. Elige qué versión usar.", icon: HardDriveDownload, view: BinariesPage },
  extensions: { label: "Extensiones PHP", subtitle: "Activa o desactiva extensiones de la versión de PHP elegida.", icon: Puzzle, view: ExtensionsPage },
  env: { label: "Variables de entorno", subtitle: "PATH y variables de tu usuario de Windows.", icon: SlidersHorizontal, view: EnvPage },
};

export default function App() {
  const { page, go } = useNav();
  const setProgress = useProgress((s) => s.set);
  const overview = useOverview();
  const { subtitle, label, view: View } = PAGES[page];

  useEffect(() => {
    const unlisten = listen<Progress>("install-progress", (e) => setProgress(e.payload));
    return () => void unlisten.then((fn) => fn());
  }, [setProgress]);

  return (
    <div className="flex h-screen">
      <aside className="flex w-56 shrink-0 flex-col gap-2 bg-base-200 p-3">
        <h1 className="px-2 py-3 text-xl font-bold">Laralite</h1>
        <ul className="menu w-full gap-1 p-0">
          {(Object.keys(PAGES) as PageId[]).map((id) => {
            const { icon: Icon, label } = PAGES[id];
            return (
              <li key={id}>
                <button className={id === page ? "menu-active" : ""} onClick={() => go(id)}>
                  <Icon size={18} /> {label}
                </button>
              </li>
            );
          })}
        </ul>
        <button
          className="btn btn-ghost btn-sm mt-auto justify-start"
          title={overview.data?.root}
          onClick={() => overview.data && api.open(overview.data.root)}
        >
          <FolderOpen size={16} /> Carpeta de datos
        </button>
      </aside>
      <main className="grow overflow-y-auto p-6">
        <header className="mb-5">
          <h2 className="text-2xl font-bold">{label}</h2>
          <p className="text-sm opacity-70">{subtitle}</p>
        </header>
        <View />
      </main>
      <Toasts />
    </div>
  );
}
