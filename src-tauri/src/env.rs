//! Variables de entorno del USUARIO (HKCU\Environment). No requiere administrador.
use crate::error::{Ctx, Res};
use serde::Serialize;
use std::path::Path;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_EXPAND_SZ};
use winreg::{RegKey, RegValue};

#[derive(Serialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

fn key(write: bool) -> Res<RegKey> {
    let flags = if write { KEY_READ | KEY_WRITE } else { KEY_READ };
    RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags("Environment", flags).ctx("registro")
}

/// Todas las variables de usuario salvo PATH (que se maneja aparte).
pub fn list() -> Res<Vec<EnvVar>> {
    let mut vars: Vec<EnvVar> = key(false)?
        .enum_values()
        .flatten()
        .filter(|(name, _)| !name.eq_ignore_ascii_case("path"))
        .filter_map(|(name, v)| Some(EnvVar { name, value: String::from_utf16_lossy(&utf16(&v.bytes)) }))
        .collect();
    vars.sort_by_key(|v| v.name.to_lowercase());
    Ok(vars)
}

pub fn set(name: &str, value: &str) -> Res<()> {
    if name.trim().is_empty() || name.contains('=') {
        return Err("Nombre de variable invalido".into());
    }
    write_expand(&key(true)?, name.trim(), value)?;
    broadcast();
    Ok(())
}

pub fn remove(name: &str) -> Res<()> {
    key(true)?.delete_value(name).ctx("borrar variable")?;
    broadcast();
    Ok(())
}

pub fn path_entries() -> Res<Vec<String>> {
    let raw: String = key(false)?.get_value("Path").unwrap_or_default();
    Ok(raw.split(';').filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_string()).collect())
}

fn write_path(entries: &[String]) -> Res<()> {
    write_expand(&key(true)?, "Path", &entries.join(";"))?;
    broadcast();
    Ok(())
}

fn same(a: &str, b: &str) -> bool {
    a.trim_end_matches('\\').eq_ignore_ascii_case(b.trim_end_matches('\\'))
}

pub fn path_contains(entry: &Path) -> bool {
    let entry = entry.to_string_lossy();
    path_entries().unwrap_or_default().iter().any(|e| same(e, &entry))
}

pub fn path_add(entry: &str) -> Res<()> {
    let mut entries = path_entries()?;
    if !entries.iter().any(|e| same(e, entry)) {
        entries.push(entry.to_string());
        write_path(&entries)?;
    }
    Ok(())
}

pub fn path_remove(entry: &str) -> Res<()> {
    let mut entries = path_entries()?;
    let before = entries.len();
    entries.retain(|e| !same(e, entry));
    if entries.len() != before {
        write_path(&entries)?;
    }
    Ok(())
}

fn write_expand(key: &RegKey, name: &str, value: &str) -> Res<()> {
    let mut bytes: Vec<u8> = value.encode_utf16().chain(std::iter::once(0)).flat_map(u16::to_le_bytes).collect();
    bytes.shrink_to_fit();
    key.set_raw_value(name, &RegValue { bytes: bytes.into(), vtype: REG_EXPAND_SZ }).ctx("escribir registro")
}

fn utf16(bytes: &[u8]) -> Vec<u16> {
    let mut v: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    while v.last() == Some(&0) {
        v.pop();
    }
    v
}

/// Avisa a Windows para que las nuevas consolas vean el cambio.
fn broadcast() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
    let param: Vec<u16> = "Environment".encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, param.as_ptr() as isize, SMTO_ABORTIFHUNG, 3000, std::ptr::null_mut());
    }
}
