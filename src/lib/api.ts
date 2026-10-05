import { invoke } from "@tauri-apps/api/core";

export type ToolId = "php" | "apache" | "mysql" | "mariadb" | "composer" | "phpmyadmin" | "postgres" | "heidisql";

export interface Release {
  id: string;
  version: string;
  variant: string | null;
  url: string;
  size: number | null;
  sha256: string | null;
}
export interface Catalog {
  fetched_at: number;
  releases: Release[];
}
export interface ToolState {
  installed: string[];
  active: string | null;
  in_path: boolean;
}
export interface Overview {
  root: string;
  tools: Record<ToolId, ToolState>;
}
export interface ServiceInfo {
  tool: ToolId;
  port: number;
  running: boolean;
  pid: number | null;
}
export interface EnvVar {
  name: string;
  value: string;
}
export interface Extension {
  name: string;
  enabled: boolean;
}

/** Unica puerta hacia el backend Rust: un wrapper por comando. */
export const api = {
  overview: () => invoke<Overview>("overview"),
  releases: (tool: ToolId, refresh = false) => invoke<Catalog>("get_releases", { tool, refresh }),
  install: (tool: ToolId, release: Release) => invoke<void>("install_release", { tool, release }),
  uninstall: (tool: ToolId, id: string) => invoke<void>("uninstall_version", { tool, id }),
  activate: (tool: ToolId, id: string) => invoke<void>("activate_version", { tool, id }),
  setInPath: (tool: ToolId, enabled: boolean) => invoke<void>("set_in_path", { tool, enabled }),
  envVars: () => invoke<EnvVar[]>("env_vars"),
  envSet: (name: string, value: string) => invoke<void>("env_set", { name, value }),
  envRemove: (name: string) => invoke<void>("env_remove", { name }),
  pathEntries: () => invoke<string[]>("path_entries"),
  pathAdd: (entry: string) => invoke<void>("path_add", { entry }),
  pathRemove: (entry: string) => invoke<void>("path_remove", { entry }),
  phpExtensions: (version: string) => invoke<Extension[]>("php_extensions", { version }),
  phpSetExtension: (version: string, name: string, enabled: boolean) =>
    invoke<void>("php_set_extension", { version, name, enabled }),
  servicesStatus: () => invoke<ServiceInfo[]>("services_status"),
  serviceStart: (tool: ToolId) => invoke<void>("service_start", { tool }),
  serviceStop: (tool: ToolId) => invoke<void>("service_stop", { tool }),
  launchHeidisql: () => invoke<void>("launch_heidisql"),
  open: (target: string) => invoke<void>("open_external", { target }),
};
