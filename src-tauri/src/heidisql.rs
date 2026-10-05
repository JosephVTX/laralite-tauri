//! HeidiSQL portable: conexiones locales ya creadas (root sin clave) y lanzador.
use crate::error::{Ctx, Res};
use crate::{install, paths, tools::Tool};
use std::path::Path;

/// (nombre, NetType de HeidiSQL, puerto, usuario). Solo se escribe si no hay configuracion previa.
const SERVERS: [(&str, u8, u16, &str); 2] = [("Laralite MySQL-MariaDB", 0, 3306, "root"), ("Laralite PostgreSQL", 10, 5432, "postgres")];

pub fn configure(dir: &Path) -> Res<()> {
    let file = dir.join("portable_settings.txt");
    let mut out = std::fs::read_to_string(&file).unwrap_or_default();
    let before = out.len();
    // Idioma por defecto: español (si el usuario ya eligio uno, se respeta).
    if !out.lines().any(|l| l.starts_with("Language<|||>")) {
        out += "Language<|||>1<|||>es\r\n";
    }
    for (name, net, port, user) in SERVERS {
        let key = format!("Servers\\{name}\\");
        // HeidiSQL reescribe el archivo y puede dejar la sesion sin Host: se repone si falta.
        if out.contains(&format!("{key}Host<|||>")) {
            continue;
        }
        if !out.is_empty() && !out.ends_with('\n') {
            out += "\r\n";
        }
        let rows = [("NetType", 3, net.to_string()), ("Host", 1, "127.0.0.1".into()), ("User", 1, user.into()), ("Password", 1, String::new()), ("Port", 1, port.to_string())];
        for (field, kind, value) in rows {
            out += &format!("{key}{field}<|||>{kind}<|||>{value}\r\n");
        }
    }
    if out.len() == before {
        return Ok(());
    }
    std::fs::write(file, out).ctx("escribir portable_settings.txt")
}

pub fn launch() -> Res<()> {
    let id = install::load_state().active.get(Tool::Heidisql.id()).cloned().ctx("Instala y activa HeidiSQL primero")?;
    let dir = paths::version_dir(Tool::Heidisql, &id);
    configure(&dir)?;
    std::process::Command::new(dir.join("heidisql.exe")).current_dir(&dir).spawn().ctx("abrir HeidiSQL")?;
    Ok(())
}
