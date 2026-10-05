use crate::error::{Ctx, Res};
use crate::tools::Tool;
use std::path::PathBuf;

/// Todo vive en `%LOCALAPPDATA%\Laralite`.
pub fn root() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("Laralite")
}

pub fn versions_dir(tool: Tool) -> PathBuf {
    root().join("bin").join(tool.id())
}

pub fn version_dir(tool: Tool, id: &str) -> PathBuf {
    versions_dir(tool).join(id)
}

/// Junction estable que apunta a la version activa; es lo que se agrega al PATH.
pub fn current_link(tool: Tool) -> PathBuf {
    root().join("current").join(tool.id())
}

/// Carpeta que va al PATH para la herramienta.
pub fn path_entry(tool: Tool) -> PathBuf {
    let sub = tool.meta().path_subdir;
    if sub.is_empty() { current_link(tool) } else { current_link(tool).join(sub) }
}

pub fn downloads_dir() -> PathBuf {
    root().join("cache").join("downloads")
}

pub fn catalog_file() -> PathBuf {
    root().join("cache").join("catalog.json")
}

/// Datos de bases de datos, separados por version (los formatos no son compatibles).
pub fn data_dir(tool: Tool, id: &str) -> PathBuf {
    root().join("data").join(tool.id()).join(id)
}

pub fn logs_dir() -> PathBuf {
    root().join("logs")
}

pub fn state_file() -> PathBuf {
    root().join("state.json")
}

pub async fn ensure(dir: &PathBuf) -> Res<()> {
    tokio::fs::create_dir_all(dir).await.ctx("No se pudo crear la carpeta")
}
