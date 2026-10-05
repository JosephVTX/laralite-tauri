//! phpMyAdmin sin login: `config.inc.php` entra directo como root (MySQL/MariaDB local sin clave).
use crate::error::{Ctx, Res};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn configure(dir: &Path) -> Res<()> {
    let tmp = dir.join("tmp");
    std::fs::create_dir_all(&tmp).ctx("crear tmp")?;
    let seed = format!("{:?}{}", SystemTime::now().duration_since(UNIX_EPOCH), std::process::id());
    let secret = &hex::encode(Sha256::digest(seed))[..32];
    let tmp = tmp.to_string_lossy().replace('\\', "/");
    let cfg = format!(
        "<?php\n$cfg['blowfish_secret'] = '{secret}';\n$i = 1;\n\
$cfg['Servers'][$i]['auth_type'] = 'config';\n\
$cfg['Servers'][$i]['host'] = '127.0.0.1';\n\
$cfg['Servers'][$i]['user'] = 'root';\n\
$cfg['Servers'][$i]['password'] = '';\n\
$cfg['Servers'][$i]['AllowNoPassword'] = true;\n\
$cfg['TempDir'] = '{tmp}';\n"
    );
    std::fs::write(dir.join("config.inc.php"), cfg).ctx("escribir config.inc.php")
}
