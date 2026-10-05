//! Iniciar/detener Apache, MySQL, MariaDB, PostgreSQL y phpMyAdmin (procesos hijos sin ventana).
use crate::error::{Ctx, Res};
use crate::{fsx, install, paths, tools::Tool};
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::net::TcpListener;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Procesos en ejecucion; clonable para usarlo dentro de `spawn_blocking`.
#[derive(Clone, Default)]
pub struct Services(Arc<Mutex<HashMap<Tool, Child>>>);

#[derive(Serialize)]
pub struct ServiceInfo {
    tool: Tool,
    port: u16,
    running: bool,
    pid: Option<u32>,
}

/// Ruta con `/` (Apache y MySQL la aceptan en argumentos y .conf sin escapes).
fn fwd(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn command(exe: PathBuf, dir: &Path) -> Command {
    let mut c = Command::new(exe);
    c.current_dir(dir).creation_flags(fsx::CREATE_NO_WINDOW);
    c
}

fn log_tail(path: &Path) -> String {
    let text = std::fs::read_to_string(path).unwrap_or_default();
    let lines: Vec<&str> = text.lines().rev().take(12).collect();
    lines.into_iter().rev().collect::<Vec<_>>().join("\n")
}

/// Apuntar `SRVROOT` al directorio real (la conf de Apache Lounge trae `c:/Apache24`).
fn ensure_apache_conf(dir: &Path) -> Res<()> {
    let conf = dir.join("conf/httpd.conf");
    let text = std::fs::read_to_string(&conf).ctx("leer httpd.conf")?;
    let srvroot = Regex::new(r#"(?m)^Define SRVROOT ".*"\s*$"#).unwrap();
    let name = Regex::new(r"(?m)^#ServerName .*$").unwrap();
    let out = srvroot.replace(&text, format!("Define SRVROOT \"{}\"", fwd(dir)).as_str());
    let out = name.replace(&out, "ServerName localhost:80");
    if out != text {
        std::fs::write(&conf, out.as_bytes()).ctx("escribir httpd.conf")?;
    }
    Ok(())
}

/// Inicializa el datadir la primera vez; devuelve su ruta.
fn ensure_datadir(tool: Tool, id: &str, dir: &Path) -> Res<PathBuf> {
    let data = paths::data_dir(tool, id);
    let marker = if tool == Tool::Postgres { "PG_VERSION" } else { "mysql" };
    if data.join(marker).exists() {
        return Ok(data);
    }
    let _ = std::fs::remove_dir_all(&data);
    // mysqld crea solo la ultima carpeta: el padre (`data/<tool>`) debe existir.
    std::fs::create_dir_all(data.parent().ctx("datadir")?).ctx("crear data")?;
    let datadir = format!("--datadir={}", fwd(&data));
    let mut cmd = match tool {
        Tool::Postgres => {
            let mut c = command(dir.join("bin/initdb.exe"), dir);
            c.args(["-D", &fwd(&data), "-U", "postgres", "-A", "trust", "-E", "UTF8"]);
            c
        }
        Tool::Mariadb => {
            let bin = dir.join("bin");
            let exe = ["mariadb-install-db.exe", "mysql_install_db.exe"].iter().map(|n| bin.join(n)).find(|p| p.exists());
            let mut c = command(exe.ctx("mariadb-install-db")?, dir);
            c.arg(&datadir);
            c
        }
        _ => {
            let mut c = command(dir.join("bin/mysqld.exe"), dir);
            c.args(["--initialize-insecure", &format!("--basedir={}", fwd(dir)), &datadir]);
            c
        }
    };
    let out = cmd.output().ctx("inicializar base de datos")?;
    if out.status.success() {
        Ok(data)
    } else {
        Err(format!("No se pudo inicializar la base de datos: {}", String::from_utf8_lossy(&out.stderr).trim()))
    }
}

fn alive(child: &mut Child) -> bool {
    matches!(child.try_wait(), Ok(None))
}

pub fn status(svc: &Services) -> Vec<ServiceInfo> {
    let mut procs = svc.0.lock().unwrap();
    Tool::ALL
        .iter()
        .filter_map(|&tool| {
            let port = tool.meta().port?;
            let child = procs.get_mut(&tool).and_then(|c| alive(c).then_some(c));
            Some(ServiceInfo { tool, port, running: child.is_some(), pid: child.map(|c| c.id()) })
        })
        .collect()
}

pub fn start(svc: &Services, tool: Tool) -> Res<()> {
    let port = tool.meta().port.ctx("no es un servicio")?;
    if status(svc).iter().any(|s| s.tool == tool && s.running) {
        return Ok(());
    }
    let id = install::load_state().active.get(tool.id()).cloned().ctx("Activa una version primero")?;
    let dir = paths::version_dir(tool, &id);
    if TcpListener::bind(("0.0.0.0", port)).is_err() {
        return Err(format!("El puerto {port} ya esta en uso"));
    }

    let mut cmd = if tool == Tool::Apache {
        ensure_apache_conf(&dir)?;
        let mut c = command(dir.join("bin/httpd.exe"), &dir);
        c.args(["-d", &fwd(&dir)]);
        c
    } else if tool == Tool::Phpmyadmin {
        // Servidor integrado de PHP: no depende de Apache.
        let php = install::load_state().active.get(Tool::Php.id()).map(|v| paths::version_dir(Tool::Php, v));
        let php = php.filter(|d| d.join("php.exe").exists()).ctx("phpMyAdmin necesita un PHP activo (Programas > PHP)")?;
        let mut c = command(php.join("php.exe"), &dir);
        c.args(["-S", &format!("127.0.0.1:{port}"), "-t", &fwd(&dir)]);
        c
    } else if tool == Tool::Postgres {
        let data = ensure_datadir(tool, &id, &dir)?;
        let mut c = command(dir.join("bin/postgres.exe"), &dir);
        c.args(["-D", &fwd(&data), "-p", &port.to_string()]);
        c
    } else {
        let data = ensure_datadir(tool, &id, &dir)?;
        let mut c = command(dir.join("bin/mysqld.exe"), &dir);
        c.args([format!("--basedir={}", fwd(&dir)), format!("--datadir={}", fwd(&data)), "--console".into()]);
        c
    };

    let log = paths::logs_dir().join(format!("{}.log", tool.id()));
    std::fs::create_dir_all(paths::logs_dir()).ctx("crear logs")?;
    let file = std::fs::File::create(&log).ctx("crear log")?;
    cmd.stdout(Stdio::from(file.try_clone().ctx("log")?)).stderr(Stdio::from(file)).stdin(Stdio::null());
    let mut child = cmd.spawn().ctx("iniciar proceso")?;

    // Si muere enseguida (config invalida, falta VC++, etc.) devolvemos el log.
    std::thread::sleep(Duration::from_millis(1500));
    if let Ok(Some(code)) = child.try_wait() {
        return Err(format!("{} se cerro (codigo {code}):\n{}", tool.id(), log_tail(&log)));
    }
    svc.0.lock().unwrap().insert(tool, child);
    Ok(())
}

fn kill_tree(pid: u32) {
    let _ = Command::new("taskkill").args(["/PID", &pid.to_string(), "/T", "/F"]).creation_flags(fsx::CREATE_NO_WINDOW).output();
}

pub fn stop(svc: &Services, tool: Tool) -> Res<()> {
    let Some(mut child) = svc.0.lock().unwrap().remove(&tool) else { return Ok(()) };
    if matches!(tool, Tool::Mysql | Tool::Mariadb | Tool::Postgres) {
        // Apagado limpio de la base de datos; si falla (p. ej. root con clave) se fuerza abajo.
        let id = install::load_state().active.get(tool.id()).cloned();
        let dir = id.as_ref().map(|id| paths::version_dir(tool, id));
        let admin = dir.as_ref().and_then(|d| {
            let names: &[&str] = if tool == Tool::Postgres { &["pg_ctl.exe"] } else { &["mysqladmin.exe", "mariadb-admin.exe"] };
            names.iter().map(|n| d.join("bin").join(n)).find(|p| p.exists())
        });
        if let (Some(exe), Some(dir), Some(id)) = (admin, dir, id) {
            let mut cmd = command(exe, &dir);
            if tool == Tool::Postgres {
                cmd.args(["stop", "-m", "fast", "-D", &fwd(&paths::data_dir(tool, &id))]);
            } else {
                cmd.args(["-u", "root", "--protocol=tcp", "shutdown"]);
            }
            let _ = cmd.output();
            let deadline = Instant::now() + Duration::from_secs(15);
            while alive(&mut child) && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(250));
            }
        }
    }
    if alive(&mut child) {
        kill_tree(child.id());
        let _ = child.wait();
    }
    Ok(())
}

pub fn stop_all(svc: &Services) {
    for tool in Tool::ALL {
        let _ = stop(svc, tool);
    }
}
