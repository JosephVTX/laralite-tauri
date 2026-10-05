import { useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";
import { ReleaseRow } from "../components/ReleaseRow";
import { api } from "../lib/api";
import { fmtDate } from "../lib/format";
import { keys, useAction, useOverview, useReleases } from "../lib/queries";
import { useNav } from "../lib/stores";
import { TOOLS } from "../lib/tools";

const PAGE = 60;

export function BinariesPage() {
  const { tool, go } = useNav();
  const meta = TOOLS.find((t) => t.id === tool)!;
  const overview = useOverview();
  const releases = useReleases(tool);
  const qc = useQueryClient();
  const [filter, setFilter] = useState("");
  const [onlyInstalled, setOnlyInstalled] = useState(false);
  const [limit, setLimit] = useState(PAGE);

  const state = overview.data?.tools[tool];
  const refresh = useAction(() => api.releases(tool, true).then((c) => qc.setQueryData(keys.releases(tool), c)), {
    invalidate: [],
    ok: "Catalogo actualizado",
  });
  const togglePath = useAction((on: boolean) => api.setInPath(tool, on));

  const rows = useMemo(() => {
    const q = filter.trim().toLowerCase();
    return (releases.data?.releases ?? []).filter(
      (r) => (!q || r.id.toLowerCase().includes(q)) && (!onlyInstalled || state?.installed.includes(r.id)),
    );
  }, [releases.data, filter, onlyInstalled, state]);

  return (
    <section className="space-y-4">
      <div role="tablist" className="tabs tabs-box">
        {TOOLS.map((t) => (
          <button
            key={t.id}
            role="tab"
            className={`tab ${t.id === tool ? "tab-active" : ""}`}
            onClick={() => {
              go("binaries", t.id);
              setLimit(PAGE);
            }}
          >
            {t.label}
            {(overview.data?.tools[t.id].installed.length ?? 0) > 0 && (
              <span className="badge badge-success badge-xs ml-2">{overview.data?.tools[t.id].installed.length}</span>
            )}
          </button>
        ))}
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <p className="grow text-sm opacity-70">{meta.hint}</p>
        {meta.path !== false && <label className="label cursor-pointer gap-2" title="Agrega la carpeta current de esta herramienta al PATH del usuario">
          <span className="text-sm">Usar desde cualquier terminal (PATH)</span>
          <input
            type="checkbox"
            className="toggle toggle-sm toggle-primary"
            checked={state?.in_path ?? false}
            disabled={togglePath.isPending}
            onChange={(e) => togglePath.mutate(e.target.checked)}
          />
        </label>}
      </div>

      <div className="flex flex-wrap items-center gap-3">
        <input
          className="input input-sm"
          placeholder="Buscar versión (ej. 8.3)"
          value={filter}
          onChange={(e) => {
            setFilter(e.target.value);
            setLimit(PAGE);
          }}
        />
        <label className="label cursor-pointer gap-2">
          <input type="checkbox" className="checkbox checkbox-sm" checked={onlyInstalled} onChange={(e) => setOnlyInstalled(e.target.checked)} />
          <span className="text-sm">Solo las que tengo</span>
        </label>
        <span className="grow" />
        {releases.data && <span className="text-xs opacity-60">Lista del: {fmtDate(releases.data.fetched_at)}</span>}
        <button className="btn btn-sm" disabled={refresh.isPending} onClick={() => refresh.mutate()}>
          {refresh.isPending && <span className="loading loading-spinner loading-xs" />} Actualizar lista
        </button>
      </div>

      {releases.isPending && (
        <div className="flex justify-center p-10">
          <span className="loading loading-lg loading-spinner" />
        </div>
      )}
      {releases.isError && (
        <div role="alert" className="alert alert-error">
          {String(releases.error)}
        </div>
      )}

      {releases.data && (
        <div className="overflow-x-auto rounded-box border border-base-300">
          <table className="table table-sm">
            <thead>
              <tr>
                <th>Versión</th>
                <th>Build</th>
                <th>Tamaño</th>
                <th>Estado</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {rows.slice(0, limit).map((r) => (
                <ReleaseRow key={r.id} tool={tool} release={r} installed={state?.installed.includes(r.id) ?? false} active={state?.active === r.id} />
              ))}
              {rows.length === 0 && (
                <tr>
                  <td colSpan={5} className="text-center opacity-60">
                    Sin resultados
                  </td>
                </tr>
              )}
            </tbody>
          </table>
          {rows.length > limit && (
            <button className="btn btn-ghost btn-sm w-full" onClick={() => setLimit(limit + PAGE)}>
              Mostrar más ({rows.length - limit})
            </button>
          )}
        </div>
      )}
    </section>
  );
}
