# Extender Laralite

## Agregar una herramienta descargable (ej. Node, Redis)
1. `tools.rs`: variante en `Tool`, entrada en `ALL` y en `meta()` (`path_subdir`, `probe`, `referer`).
2. `catalog.rs`: funcion proveedora `async fn x() -> Res<Vec<Release>>` y rama en el `match` de `get`. Usa `Release::new` y `push_unique`.
3. `install.rs` solo cambia si el artefacto no es `.zip` (ver el caso composer.phar) o necesita post-instalacion (ver `php_ext::ensure_ini`).
4. Frontend: agregar el id a `ToolId` en `lib/api.ts` y una linea en `lib/tools.ts`. La UI de Binarios es generica.

## Agregar un comando
1. Logica en el modulo correspondiente; wrapper fino `#[tauri::command]` en `commands.rs`.
2. Registrar en `generate_handler!` de `main.rs`.
3. Wrapper tipado en `api` (`lib/api.ts`). Argumentos en camelCase desde JS (Tauri los convierte a snake_case).
4. Consumirlo con `useQuery` (usa `keys` de `lib/queries.ts`) o `useAction`.

## Agregar una pagina
Crear `features/XPage.tsx` y anadirla al arreglo `PAGES` de `App.tsx`.

## Notas de comportamiento
- IDs de version: `8.4.1-nts` (PHP, con variante) o `8.4.1`. Son el nombre de carpeta en `bin/<tool>/`.
- Eventos: `install-progress` `{tool, id, phase: downloading|extracting|done, downloaded, total}`.
- Al instalar PHP se crea `php.ini` desde `php.ini-development`, `extension_dir="ext"` y se activan las extensiones de Laravel (`DEFAULTS` en `php_ext.rs`).
- PATH/variables escriben `REG_EXPAND_SZ` en `HKCU\Environment` y avisan con `WM_SETTINGCHANGE`; las consolas ya abiertas no ven el cambio.
