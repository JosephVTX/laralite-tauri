//! Gestion de extensiones PHP editando php.ini (las DLL ya vienen en `ext/` de cada version).
use crate::error::{Ctx, Res};
use crate::{paths, tools::Tool};
use serde::Serialize;
use std::path::Path;

/// Extensiones que Laravel necesita; se activan al instalar una version nueva.
const DEFAULTS: [&str; 10] =
    ["curl", "fileinfo", "mbstring", "openssl", "pdo_mysql", "mysqli", "zip", "intl", "pdo_sqlite", "sodium"];
/// Se cargan con `zend_extension=`.
const ZEND: [&str; 2] = ["opcache", "xdebug"];

#[derive(Serialize)]
pub struct Extension {
    pub name: String,
    pub enabled: bool,
}

/// Crea php.ini desde php.ini-development si no existe y activa las extensiones por defecto.
pub fn ensure_ini(dir: &Path) -> Res<()> {
    let ini = dir.join("php.ini");
    if ini.exists() {
        return Ok(());
    }
    let src = std::fs::read_to_string(dir.join("php.ini-development")).ctx("php.ini-development")?;
    std::fs::write(&ini, src).ctx("crear php.ini")?;
    set_line_value(&ini, "extension_dir", "\"ext\"")?;
    for name in DEFAULTS.iter().filter(|n| dir.join("ext").join(format!("php_{n}.dll")).exists()) {
        set_enabled(dir, name, true)?;
    }
    Ok(())
}

fn ini_path(version: &str) -> Res<std::path::PathBuf> {
    let dir = paths::version_dir(Tool::Php, version);
    ensure_ini(&dir)?;
    Ok(dir.join("php.ini"))
}

/// Interpreta `;extension=php_x.dll` / `zend_extension=x` -> (nombre, activa).
fn parse(line: &str) -> Option<(String, bool)> {
    let trimmed = line.trim_start();
    let (enabled, body) = match trimmed.strip_prefix(';') {
        Some(rest) => (false, rest.trim_start()),
        None => (true, trimmed),
    };
    let (key, value) = body.split_once('=')?;
    if !matches!(key.trim(), "extension" | "zend_extension") {
        return None;
    }
    let value = value.split(';').next()?.trim().trim_matches('"');
    let file = value.rsplit(['/', '\\']).next()?;
    let name = file.strip_suffix(".dll").unwrap_or(file).trim_start_matches("php_");
    (!name.is_empty()).then(|| (name.to_string(), enabled))
}

pub fn list(version: &str) -> Res<Vec<Extension>> {
    let ini = std::fs::read_to_string(ini_path(version)?).ctx("leer php.ini")?;
    let enabled: Vec<String> = ini.lines().filter_map(parse).filter(|(_, on)| *on).map(|(n, _)| n).collect();
    let ext_dir = paths::version_dir(Tool::Php, version).join("ext");
    let mut out: Vec<Extension> = std::fs::read_dir(ext_dir)
        .ctx("carpeta ext")?
        .flatten()
        .filter_map(|e| {
            let file = e.file_name().into_string().ok()?;
            let name = file.strip_suffix(".dll")?.trim_start_matches("php_").to_string();
            Some(Extension { enabled: enabled.contains(&name), name })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub fn set_enabled(dir: &Path, name: &str, on: bool) -> Res<()> {
    let ini = dir.join("php.ini");
    let text = std::fs::read_to_string(&ini).ctx("leer php.ini")?;
    let mut found = false;
    let mut lines: Vec<String> = text
        .lines()
        .map(|line| match parse(line) {
            Some((n, active)) if n == name => {
                let body = line.trim_start().trim_start_matches(';').trim_start().to_string();
                match (on, active, found) {
                    (true, _, false) => {
                        found = true;
                        body
                    }
                    (true, _, true) => line.to_string(),
                    (false, true, _) => format!(";{body}"),
                    (false, false, _) => line.to_string(),
                }
            }
            _ => line.to_string(),
        })
        .collect();
    if on && !found {
        let key = if ZEND.contains(&name) { "zend_extension" } else { "extension" };
        lines.push(format!("{key}={name}"));
    }
    std::fs::write(&ini, lines.join("\r\n") + "\r\n").ctx("escribir php.ini")
}

pub fn set_extension(version: &str, name: &str, on: bool) -> Res<()> {
    ini_path(version)?;
    set_enabled(&paths::version_dir(Tool::Php, version), name, on)
}

/// Reemplaza `key = ...` (comentado o no) por `key = value`.
fn set_line_value(ini: &Path, key: &str, value: &str) -> Res<()> {
    let text = std::fs::read_to_string(ini).ctx("leer php.ini")?;
    let mut done = false;
    let lines: Vec<String> = text
        .lines()
        .map(|line| {
            let body = line.trim_start().trim_start_matches(';').trim_start();
            let is_key = body.split_once('=').is_some_and(|(k, _)| k.trim() == key);
            if is_key && !done {
                done = true;
                format!("{key} = {value}")
            } else {
                line.to_string()
            }
        })
        .collect();
    std::fs::write(ini, lines.join("\r\n") + "\r\n").ctx("escribir php.ini")
}
