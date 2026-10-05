import { Download, FolderOpen, Globe, Play, Square } from "lucide-react";
import { api, type ServiceInfo } from "../lib/api";
import { keys, useAction, useOverview, useServices } from "../lib/queries";
import { useNav } from "../lib/stores";
import { toolLabel } from "../lib/tools";

const ABOUT: Record<string, string> = { apache: "Servidor web", mysql: "Base de datos", mariadb: "Base de datos (alternativa a MySQL)",
  postgres: "Base de datos PostgreSQL (usuario postgres, sin clave)",
  phpmyadmin: "Administrador de bases de datos (entra solo, sin clave)",
};

function ServiceCard({ s }: { s: ServiceInfo }) {
  const overview = useOverview();
  const go = useNav((n) => n.go);
  const state = overview.data?.tools[s.tool];
  const installed = state?.installed ?? [];
  const active = state?.active ?? "";
  const opts = { invalidate: [keys.services] };
  const start = useAction(api.serviceStart, opts);
  const stop = useAction(api.serviceStop, opts);
  const activate = useAction((id: string) => api.activate(s.tool, id));
  const busy = start.isPending || stop.isPending;
  const root = overview.data?.root;

  return (
    <div className={`card bg-base-200 shadow-sm ${s.running ? "ring-2 ring-success/60" : ""}`}>
      <div className="card-body gap-3">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="card-title">{toolLabel(s.tool)}</h3>
            <p className="text-xs opacity-60">{ABOUT[s.tool]}</p>
          </div>
          <span className={`badge ${s.running ? "badge-success" : "badge-ghost"}`}>{s.running ? "Encendido" : "Apagado"}</span>
        </div>

        {installed.length === 0 ? (
          <button className="btn btn-primary" onClick={() => go("binaries", s.tool)}>
            <Download size={16} /> Instalar {toolLabel(s.tool)}
          </button>
        ) : (
          <>
            <label className="flex items-center gap-2 text-sm">
              <span className="opacity-70">Versión</span>
              <select
                className="select select-sm grow"
                value={active}
                disabled={s.running || activate.isPending}
                title={s.running ? "Apágalo para cambiar de versión" : undefined}
                onChange={(e) => activate.mutate(e.target.value)}
              >
                {!active && <option value="">Elige una…</option>}
                {installed.map((v) => (
                  <option key={v}>{v}</option>
                ))}
              </select>
            </label>
            {s.running ? (
              <button className="btn btn-error" disabled={busy} onClick={() => stop.mutate(s.tool)}>
                {stop.isPending ? <span className="loading loading-spinner loading-sm" /> : <Square size={16} />} Apagar
              </button>
            ) : (
              <button className="btn btn-success" disabled={busy || !active} onClick={() => start.mutate(s.tool)}>
                {start.isPending ? <span className="loading loading-spinner loading-sm" /> : <Play size={16} />} Encender
              </button>
            )}
          </>
        )}

        <div className="flex items-center gap-1 text-xs opacity-70">
          <span className="grow">Puerto {s.port}</span>
          {s.tool === "apache" && s.running && (
            <button className="btn btn-ghost btn-xs" onClick={() => api.open("http://localhost")}>
              <Globe size={14} /> Abrir sitio
            </button>
          )}
          {s.tool === "phpmyadmin" && s.running && (
            <button className="btn btn-primary btn-xs" onClick={() => api.open(`http://127.0.0.1:${s.port}`)}>
              <Globe size={14} /> Abrir phpMyAdmin
            </button>
          )}
          {s.tool === "apache" && active && root && (
            <button className="btn btn-ghost btn-xs" onClick={() => api.open(`${root}\\bin\\apache\\${active}\\htdocs`)}>
              <FolderOpen size={14} /> Carpeta web
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function HeidiCard() {
  const overview = useOverview();
  const go = useNav((n) => n.go);
  const open = useAction(api.launchHeidisql);
  const state = overview.data?.tools.heidisql;
  const ready = !!state?.active;

  return (
    <div className="card bg-base-200 shadow-sm">
      <div className="card-body gap-3">
        <div>
          <h3 className="card-title">HeidiSQL</h3>
          <p className="text-xs opacity-60">Cliente gráfico para MySQL, MariaDB y PostgreSQL (conexiones locales ya creadas)</p>
        </div>
        {ready ? (
          <button className="btn btn-primary" disabled={open.isPending} onClick={() => open.mutate(undefined)}>
            <Globe size={16} /> Abrir HeidiSQL
          </button>
        ) : (
          <button className="btn btn-primary" onClick={() => go("binaries", "heidisql")}>
            <Download size={16} /> Instalar HeidiSQL
          </button>
        )}
      </div>
    </div>
  );
}

export function ServicesPage() {
  const services = useServices();
  const overview = useOverview();
  const go = useNav((n) => n.go);
  const nothing = overview.data && Object.values(overview.data.tools).every((t) => t.installed.length === 0);

  return (
    <section className="space-y-5">
      {nothing && (
        <div role="alert" className="alert alert-info">
          <span>¡Bienvenido! Empieza descargando PHP y un servidor desde “Programas”.</span>
          <button className="btn btn-sm" onClick={() => go("binaries")}>
            Ir a Programas
          </button>
        </div>
      )}
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">{services.data?.map((s) => <ServiceCard key={s.tool} s={s} />)}
        <HeidiCard /></div>
      <p className="text-xs opacity-60">Al cerrar Laralite, los servicios encendidos se apagan solos.</p>
    </section>
  );
}
