import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { AddForm } from "../components/AddForm";
import { api } from "../lib/api";
import { keys, useAction } from "../lib/queries";

/** Fila con boton de quitar; comun a PATH y variables. */
function Row({ children, onRemove, busy }: { children: ReactNode; onRemove: () => void; busy: boolean }) {
  return (
    <li className="list-row items-center">
      <div className="list-col-grow min-w-0 break-all font-mono text-sm">{children}</div>
      <button className="btn btn-xs btn-error btn-outline" disabled={busy} onClick={onRemove}>
        Quitar
      </button>
    </li>
  );
}

export function EnvPage() {
  const vars = useQuery({ queryKey: keys.env, queryFn: api.envVars });
  const path = useQuery({ queryKey: keys.path, queryFn: api.pathEntries });

  const setVar = useAction(([n, v]: string[]) => api.envSet(n, v), { invalidate: [keys.env], ok: "Variable guardada" });
  const delVar = useAction((n: string) => api.envRemove(n), { invalidate: [keys.env] });
  const addPath = useAction(([e]: string[]) => api.pathAdd(e), { invalidate: [keys.path, keys.overview], ok: "Agregado al PATH" });
  const delPath = useAction((e: string) => api.pathRemove(e), { invalidate: [keys.path, keys.overview] });

  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <section className="space-y-3">
        <h2 className="text-lg font-semibold">PATH (usuario)</h2>
        <AddForm fields={["C:\\ruta\\a\\agregar"]} busy={addPath.isPending} onSubmit={(v) => addPath.mutate(v)} />
        <ul className="list rounded-box bg-base-200">
          {path.data?.map((e) => (
            <Row key={e} busy={delPath.isPending} onRemove={() => delPath.mutate(e)}>
              {e}
            </Row>
          ))}
        </ul>
      </section>
      <section className="space-y-3">
        <h2 className="text-lg font-semibold">Variables de usuario</h2>
        <AddForm fields={["NOMBRE", "valor"]} busy={setVar.isPending} onSubmit={(v) => setVar.mutate(v)} />
        <ul className="list rounded-box bg-base-200">
          {vars.data?.map((v) => (
            <Row key={v.name} busy={delVar.isPending} onRemove={() => delVar.mutate(v.name)}>
              <b>{v.name}</b>={v.value}
            </Row>
          ))}
          {vars.data?.length === 0 && <li className="list-row opacity-60">Sin variables</li>}
        </ul>
      </section>
      <p className="text-xs opacity-60 lg:col-span-2">Los cambios aplican a consolas nuevas; las ya abiertas deben reiniciarse.</p>
    </div>
  );
}
