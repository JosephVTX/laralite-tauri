import { QueryKey, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, type ToolId } from "./api";
import { useToasts } from "./stores";

export const keys = {
  overview: ["overview"] as const,
  releases: (tool: ToolId) => ["releases", tool] as const,
  services: ["services"] as const,
  env: ["env"] as const,
  path: ["path"] as const,
  extensions: (version: string) => ["extensions", version] as const,
};

export const useOverview = () => useQuery({ queryKey: keys.overview, queryFn: api.overview });

/** Estado local de procesos (sin red): se refresca cada 3 s por si un servicio se cae. */
export const useServices = () => useQuery({ queryKey: keys.services, queryFn: api.servicesStatus, refetchInterval: 3000 });

/** Una sola consulta por herramienta (staleTime infinito + cache en disco del backend). */
export const useReleases = (tool: ToolId) => useQuery({ queryKey: keys.releases(tool), queryFn: () => api.releases(tool) });

/** Mutacion estandar: avisa errores/exito con toast e invalida las queries indicadas. */
export function useAction<A>(fn: (arg: A) => Promise<unknown>, opts: { invalidate?: QueryKey[]; ok?: string } = {}) {
  const qc = useQueryClient();
  const push = useToasts((s) => s.push);
  return useMutation({
    mutationFn: fn,
    onSuccess: () => {
      (opts.invalidate ?? [keys.overview]).forEach((queryKey) => qc.invalidateQueries({ queryKey }));
      if (opts.ok) push("success", opts.ok);
    },
    onError: (e) => push("error", String(e)),
  });
}
