use std::fmt::Display;

/// Resultado de todos los comandos: el error viaja al frontend como texto.
pub type Res<T> = Result<T, String>;

pub trait Ctx<T> {
    fn ctx(self, msg: &str) -> Res<T>;
}

impl<T, E: Display> Ctx<T> for Result<T, E> {
    fn ctx(self, msg: &str) -> Res<T> {
        self.map_err(|e| format!("{msg}: {e}"))
    }
}

impl<T> Ctx<T> for Option<T> {
    fn ctx(self, msg: &str) -> Res<T> {
        self.ok_or_else(|| format!("{msg}: formato inesperado"))
    }
}
