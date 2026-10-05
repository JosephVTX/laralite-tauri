import { useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { api } from "../lib/api";
import { keys, useAction, useOverview } from "../lib/queries";
import { useNav } from "../lib/stores";

export function ExtensionsPage() {
  const overview = useOverview();
  const go = useNav((n) => n.go);
  const versions = overview.data?.tools.php.installed ?? [];
  const [picked, setPicked] = useState<string>();
  const version = picked ?? overview.data?.tools.php.active ?? versions[0];
  const [filter, setFilter] = useState("");

  const exts = useQuery({ queryKey: keys.extensions(version ?? ""), queryFn: () => api.phpExtensions(version!), enabled: !!version });
  const toggle = useAction(([name, on]: [string, boolean]) => api.phpSetExtension(version!, name, on), {
    invalidate: [keys.extensions(version ?? "")],
  });

  if (!version) {
    return (
      <div role="alert" className="alert">
        <span>Primero instala una versión de PHP.</span>
        <button className="btn btn-sm btn-primary" onClick={() => go("binaries", "php")}>
          Instalar PHP
        </button>
      </div>
    );
  }
  const rows = exts.data?.filter((e) => e.name.toLowerCase().includes(filter.toLowerCase())) ?? [];

  return (
    <section className="space-y-4">
      <div className="flex flex-wrap items-center gap-3">
        <select className="select select-sm w-auto" value={version} onChange={(e) => setPicked(e.target.value)}>
          {versions.map((v) => (
            <option key={v}>{v}</option>
          ))}
        </select>
        <input className="input input-sm" placeholder="Buscar extensión" value={filter} onChange={(e) => setFilter(e.target.value)} />
        <span className="text-xs opacity-60">
          {exts.data?.filter((e) => e.enabled).length ?? 0} activas · se guarda en el php.ini de esta versión
        </span>
      </div>
      {exts.isError && (
        <div role="alert" className="alert alert-error">
          {String(exts.error)}
        </div>
      )}
      <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
        {rows.map((e) => (
          <label key={e.name} className="flex cursor-pointer items-center justify-between rounded-box bg-base-200 px-3 py-2">
            <span className="font-mono text-sm">{e.name}</span>
            <input
              type="checkbox"
              className="toggle toggle-sm toggle-primary"
              checked={e.enabled}
              disabled={toggle.isPending}
              onChange={(ev) => toggle.mutate([e.name, ev.target.checked])}
            />
          </label>
        ))}
      </div>
    </section>
  );
}
