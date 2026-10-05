use crate::error::{Ctx, Res};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;
use std::process::Command;
use std::os::windows::process::CommandExt;

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn read_json<T: DeserializeOwned + Default>(path: &Path) -> T {
    std::fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Res<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ctx("crear carpeta")?;
    }
    let text = serde_json::to_string_pretty(value).ctx("serializar")?;
    std::fs::write(path, text).ctx("escribir archivo")
}

/// Crea (o reemplaza) una junction `link -> target`. No requiere permisos de administrador.
pub fn set_junction(link: &Path, target: &Path) -> Res<()> {
    remove_junction(link)?;
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent).ctx("crear carpeta")?;
    }
    let out = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ctx("mklink")?;
    if out.status.success() { Ok(()) } else { Err(String::from_utf8_lossy(&out.stderr).trim().to_string()) }
}

/// `remove_dir` borra la junction sin tocar el destino.
pub fn remove_junction(link: &Path) -> Res<()> {
    if link.symlink_metadata().is_ok() {
        std::fs::remove_dir(link).ctx("quitar junction")?;
    }
    Ok(())
}

/// Extrae un zip en `dest`; si el zip trae una unica carpeta raiz, la aplana.
pub fn extract_zip(zip_path: &Path, dest: &Path, probe: &str) -> Res<()> {
    let tmp = dest.with_extension("tmp");
    let _ = std::fs::remove_dir_all(&tmp);
    let file = std::fs::File::open(zip_path).ctx("abrir zip")?;
    let mut archive = zip::ZipArchive::new(file).ctx("leer zip")?;
    archive.extract(&tmp).ctx("extraer zip")?;
    let entries: Vec<_> = std::fs::read_dir(&tmp).ctx("leer destino")?.flatten().collect();
    // La raiz util es la propia carpeta o la subcarpeta que contiene `probe`
    // (p. ej. Apache Lounge trae `Apache24/` junto a ReadMe.txt).
    let src = std::iter::once(tmp.clone())
        .chain(entries.iter().map(|e| e.path()).filter(|p| p.is_dir()))
        .find(|p| p.join(probe).exists())
        .unwrap_or_else(|| match entries.as_slice() {
            [only] if only.path().is_dir() => only.path(),
            _ => tmp.clone(),
        });
    let _ = std::fs::remove_dir_all(dest);
    std::fs::rename(&src, dest).ctx("mover version")?;
    let _ = std::fs::remove_dir_all(&tmp);
    Ok(())
}
