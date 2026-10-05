import { api, type Release, type ToolId } from "../lib/api";
import { fmtBytes } from "../lib/format";
import { useAction } from "../lib/queries";
import { progressKey, useProgress } from "../lib/stores";

interface Props {
  tool: ToolId;
  release: Release;
  installed: boolean;
  active: boolean;
}

export function ReleaseRow({ tool, release, installed, active }: Props) {
  const progress = useProgress((s) => s.byKey[progressKey(tool, release.id)]);
  const clear = useProgress((s) => s.clear);
  const install = useAction(() => api.install(tool, release), { ok: `${release.id} instalado` });
  const uninstall = useAction(() => api.uninstall(tool, release.id), { ok: `${release.id} eliminado` });
  const activate = useAction(() => api.activate(tool, release.id), { ok: `${release.id} ahora está en uso` });
  const busy = install.isPending || uninstall.isPending || activate.isPending;
  const pct = progress && progress.total ? Math.round((progress.downloaded / progress.total) * 100) : undefined;

  return (
    <tr className={active ? "bg-success/10" : undefined}>
      <td className="font-mono">{release.version}</td>
      <td>{release.variant && <span className="badge badge-ghost uppercase">{release.variant}</span>}</td>
      <td>{fmtBytes(release.size)}</td>
      <td className="w-48">
        {install.isPending ? (
          <div className="flex items-center gap-2 text-xs">
            <progress className="progress progress-primary" value={pct} max={100} />
            {progress?.phase === "extracting" ? "instalando…" : pct !== undefined ? `${pct}%` : "..."}
          </div>
        ) : active ? (
          <span className="badge badge-success">En uso</span>
        ) : installed ? (
          <span className="badge badge-info">Instalada</span>
        ) : null}
      </td>
      <td className="text-right">
        <div className="join">
          {!installed && (
            <button
              className="btn btn-xs btn-primary join-item"
              disabled={busy}
              onClick={() => install.mutate(undefined, { onSettled: () => clear(tool, release.id) })}
            >
              Instalar
            </button>
          )}
          {installed && !active && (
            <button className="btn btn-xs btn-success join-item" disabled={busy} onClick={() => activate.mutate()}>
              Usar esta
            </button>
          )}
          {installed && (
            <button className="btn btn-xs btn-error btn-outline join-item" disabled={busy} onClick={() => confirm(`¿Eliminar ${release.id}? Se borrarán sus archivos.`) && uninstall.mutate()}>
              Eliminar
            </button>
          )}
        </div>
      </td>
    </tr>
  );
}
