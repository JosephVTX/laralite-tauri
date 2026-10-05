//! Descarga, extraccion, activacion (junction) y desinstalacion de versiones.
use crate::catalog::Release;
use crate::error::{Ctx, Res};
use crate::{fsx, http, paths, heidisql, php_ext, phpmyadmin, tools::Tool};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    /// tool id -> version id activa
    pub active: HashMap<String, String>,
}

pub fn load_state() -> State {
    fsx::read_json(&paths::state_file())
}

#[derive(Clone, Serialize)]
struct Progress<'a> {
    tool: Tool,
    id: &'a str,
    phase: &'a str,
    downloaded: u64,
    total: u64,
}

pub fn installed(tool: Tool) -> Vec<String> {
    let Ok(dir) = std::fs::read_dir(paths::versions_dir(tool)) else { return vec![] };
    let mut ids: Vec<String> = dir
        .flatten()
        .filter(|e| e.path().join(tool.meta().probe).exists())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    ids.sort();
    ids
}

pub async fn install(app: &AppHandle, tool: Tool, rel: &Release) -> Res<()> {
    let emit = |phase: &str, downloaded: u64, total: u64| {
        let _ = app.emit("install-progress", Progress { tool, id: &rel.id, phase, downloaded, total });
    };
    let dest = paths::version_dir(tool, &rel.id);
    let is_zip = rel.url.ends_with(".zip");
    let file_name = if is_zip { format!("{}-{}.zip", tool.id(), rel.id) } else { "composer.phar".to_string() };
    paths::ensure(&paths::downloads_dir()).await?;
    let part = paths::downloads_dir().join(format!("{file_name}.part"));

    let resp = http::get(&rel.url, tool.meta().referer).send().await.ctx("descarga")?.error_for_status().ctx("descarga")?;
    let total = resp.content_length().or(rel.size).unwrap_or(0);
    let mut out = tokio::fs::File::create(&part).await.ctx("crear archivo")?;
    let mut hasher = Sha256::new();
    let (mut done, mut last_emit) = (0u64, 0u64);
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.ctx("descarga")?;
        hasher.update(&chunk);
        out.write_all(&chunk).await.ctx("escribir")?;
        done += chunk.len() as u64;
        if done - last_emit > 256 * 1024 {
            last_emit = done;
            emit("downloading", done, total);
        }
    }
    out.flush().await.ctx("escribir")?;
    drop(out);

    if let Some(expected) = &rel.sha256 {
        if !hex::encode(hasher.finalize()).eq_ignore_ascii_case(expected) {
            let _ = tokio::fs::remove_file(&part).await;
            return Err("El sha256 no coincide: descarga corrupta".into());
        }
    }

    emit("extracting", done, total);
    if is_zip {
        let (zip_path, dest) = (part.clone(), dest.clone());
        let probe = tool.meta().probe;
        tokio::task::spawn_blocking(move || fsx::extract_zip(&zip_path, &dest, probe)).await.ctx("tarea")??;
        let _ = tokio::fs::remove_file(&part).await;
    } else {
        paths::ensure(&dest).await?;
        tokio::fs::rename(&part, dest.join(tool.meta().probe)).await.ctx("mover")?;
        tokio::fs::write(dest.join("composer.bat"), "@php \"%~dp0composer.phar\" %*\r\n").await.ctx("composer.bat")?;
    }
    match tool {
        Tool::Php => php_ext::ensure_ini(&dest)?,
        Tool::Phpmyadmin => phpmyadmin::configure(&dest)?,
        Tool::Heidisql => heidisql::configure(&dest)?,
        _ => {}
    }
    emit("done", done, total);
    // Primera version instalada: queda en uso sin pedirle otro paso al usuario.
    if !load_state().active.contains_key(tool.id()) {
        set_active(tool, &rel.id)?;
    }
    Ok(())
}

/// Apunta la junction `current/<tool>` a la version elegida.
pub fn set_active(tool: Tool, id: &str) -> Res<()> {
    let dir = paths::version_dir(tool, id);
    if !dir.join(tool.meta().probe).exists() {
        return Err(format!("{} {id} no esta instalado", tool.id()));
    }
    fsx::set_junction(&paths::current_link(tool), &dir)?;
    let mut state = load_state();
    state.active.insert(tool.id().into(), id.into());
    fsx::write_json(&paths::state_file(), &state)
}

pub fn uninstall(tool: Tool, id: &str) -> Res<()> {
    let mut state = load_state();
    if state.active.get(tool.id()).is_some_and(|a| a == id) {
        fsx::remove_junction(&paths::current_link(tool))?;
        state.active.remove(tool.id());
        fsx::write_json(&paths::state_file(), &state)?;
    }
    std::fs::remove_dir_all(paths::version_dir(tool, id)).ctx("borrar version")
}
