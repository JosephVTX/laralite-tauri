use serde::{Deserialize, Serialize};

/// Binarios gestionados. Para agregar uno nuevo: variante + `meta()` + proveedor en `catalog.rs`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tool {
    Php,
    Apache,
    Mysql,
    Mariadb,
    Composer,
    Phpmyadmin,
    Postgres,
    Heidisql,
}

pub struct Meta {
    pub id: &'static str,
    /// Subcarpeta (dentro de la version) que va al PATH.
    pub path_subdir: &'static str,
    /// Archivo que prueba que la instalacion es valida.
    pub probe: &'static str,
    /// Algunos hosts exigen Referer para descargar.
    pub referer: Option<&'static str>,
    /// Puerto TCP si la herramienta es un servicio que se puede iniciar.
    pub port: Option<u16>,
}

impl Tool {
    pub const ALL: [Tool; 8] = [
        Tool::Php,
        Tool::Apache,
        Tool::Mysql,
        Tool::Mariadb,
        Tool::Postgres,
        Tool::Composer,
        Tool::Phpmyadmin,
        Tool::Heidisql,
    ];

    pub fn meta(self) -> Meta {
        match self {
            Tool::Php => Meta { id: "php", path_subdir: "", probe: "php.exe", referer: None, port: None },
            Tool::Apache => Meta {
                id: "apache",
                path_subdir: "bin",
                probe: "bin/httpd.exe",
                referer: Some("https://www.apachelounge.com/download/"),
                port: Some(80),
            },
            Tool::Mysql => Meta { id: "mysql", path_subdir: "bin", probe: "bin/mysqld.exe", referer: None, port: Some(3306) },
            Tool::Mariadb => Meta { id: "mariadb", path_subdir: "bin", probe: "bin/mysqld.exe", referer: None, port: Some(3306) },
            Tool::Composer => Meta { id: "composer", path_subdir: "", probe: "composer.phar", referer: None, port: None },
            Tool::Postgres => Meta { id: "postgres", path_subdir: "bin", probe: "bin/postgres.exe", referer: None, port: Some(5432) },
            Tool::Heidisql => Meta { id: "heidisql", path_subdir: "", probe: "heidisql.exe", referer: None, port: None },
            Tool::Phpmyadmin => Meta { id: "phpmyadmin", path_subdir: "", probe: "index.php", referer: None, port: Some(8081) },
        }
    }

    pub fn id(self) -> &'static str {
        self.meta().id
    }
}
