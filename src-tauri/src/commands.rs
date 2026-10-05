use crate::catalog::{self, Cached, Release};
use crate::env::{self, EnvVar};
use crate::error::Res;
use crate::php_ext::{self, Extension};
use crate::tools::Tool;
use crate::services::{self, ServiceInfo, Services};
use crate::{install, paths};
use serde::Serialize;
use std::collections::HashMap;
use tauri::{AppHandle, State};

#[derive(Serialize)]
pub struct ToolState {
    installed: Vec<String>,
    active: Option<String>,
    in_path: bool,
}

#[derive(Serialize)]
pub struct Overview {
    root: String,
    tools: HashMap<&'static str, ToolState>,
}

#[tauri::command]
pub fn overview() -> Overview {
    let state = install::load_state();
    let tools = Tool::ALL
        .iter()
        .map(|&t| {
            let st = ToolState {
                installed: install::installed(t),
                active: state.active.get(t.id()).cloned(),
                in_path: env::path_contains(&paths::path_entry(t)),
            };
            (t.id(), st)
        })
        .collect();
    Overview { root: paths::root().to_string_lossy().into(), tools }
}

#[tauri::command]
pub async fn get_releases(tool: Tool, refresh: bool) -> Res<Cached> {
    catalog::get(tool, refresh).await
}

#[tauri::command]
pub async fn install_release(app: AppHandle, tool: Tool, release: Release) -> Res<()> {
    install::install(&app, tool, &release).await
}

#[tauri::command]
pub fn uninstall_version(tool: Tool, id: String) -> Res<()> {
    install::uninstall(tool, &id)
}

#[tauri::command]
pub fn activate_version(tool: Tool, id: String) -> Res<()> {
    install::set_active(tool, &id)
}

#[tauri::command]
pub fn set_in_path(tool: Tool, enabled: bool) -> Res<()> {
    let entry = paths::path_entry(tool).to_string_lossy().into_owned();
    if enabled { env::path_add(&entry) } else { env::path_remove(&entry) }
}

#[tauri::command]
pub fn env_vars() -> Res<Vec<EnvVar>> {
    env::list()
}

#[tauri::command]
pub fn env_set(name: String, value: String) -> Res<()> {
    env::set(&name, &value)
}

#[tauri::command]
pub fn env_remove(name: String) -> Res<()> {
    env::remove(&name)
}

#[tauri::command]
pub fn path_entries() -> Res<Vec<String>> {
    env::path_entries()
}

#[tauri::command]
pub fn path_add(entry: String) -> Res<()> {
    env::path_add(&entry)
}

#[tauri::command]
pub fn path_remove(entry: String) -> Res<()> {
    env::path_remove(&entry)
}

#[tauri::command]
pub fn php_extensions(version: String) -> Res<Vec<Extension>> {
    php_ext::list(&version)
}

#[tauri::command]
pub fn php_set_extension(version: String, name: String, enabled: bool) -> Res<()> {
    php_ext::set_extension(&version, &name, enabled)
}

#[tauri::command]
pub fn launch_heidisql() -> Res<()> {
    crate::heidisql::launch()
}

#[tauri::command]
/// Abre una carpeta o URL con el programa predeterminado de Windows.
pub fn open_external(target: String) {
    let _ = std::process::Command::new("explorer").arg(target).spawn();
}

#[tauri::command]
pub fn services_status(svc: State<Services>) -> Vec<ServiceInfo> {
    services::status(&svc)
}

#[tauri::command]
pub async fn service_start(svc: State<'_, Services>, tool: Tool) -> Res<()> {
    let svc = svc.inner().clone();
    tokio::task::spawn_blocking(move || services::start(&svc, tool)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn service_stop(svc: State<'_, Services>, tool: Tool) -> Res<()> {
    let svc = svc.inner().clone();
    tokio::task::spawn_blocking(move || services::stop(&svc, tool)).await.map_err(|e| e.to_string())?
}
