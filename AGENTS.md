# Laralite — reglas para IA

Clon ligero de Laragon, **solo Windows**. Tauri 2 + React 19 + TypeScript + Tailwind 4 + daisyUI 5.
UI y mensajes en español.

## Reglas duras
- **Solo `pnpm`**. Nunca `npm`/`npx`/`yarn`. Usa `pnpm dlx` si hace falta ejecutar algo suelto, `pnpm tauri ...` para el CLI.
- **No repetir codigo**: antes de escribir algo, busca si ya existe un helper (ver "Arquitectura"). Si lo vas a usar 2 veces, extraelo.
- **Red una sola vez**: los catalogos de binarios se consultan una vez y se guardan en `%LOCALAPPDATA%\Laralite\cache\catalog.json`. Solo el boton "Actualizar lista" (`refresh = true`) vuelve a la red. No agregues fetchs automaticos, polling ni `refetchOnWindowFocus`.
- El unico polling permitido es `useServices` (estado local de procesos, sin red).
- Windows 100%: se permite `winreg`, junctions, `explorer`; no escribas codigo multiplataforma.
- Sin permisos de administrador: solo PATH/variables del **usuario** (HKCU) y junctions (no symlinks).
- Rust: errores como `Res<T>` (= `Result<T, String>`) con `.ctx("mensaje")`. Los comandos Tauri viven en `commands.rs` y son finos; la logica va en su modulo.
- Frontend: todo acceso al backend pasa por `src/lib/api.ts`; mutaciones por `useAction` (toast + invalidacion); estado local compartido con zustand (`lib/stores.ts`).
- daisyUI 5 / Tailwind 4: `@import "tailwindcss"; @plugin "daisyui"` en `src/index.css`; no hay `tailwind.config.js`.

## Entorno de build (Windows)
- Requiere toolchain Rust **MSVC** + *Visual Studio Build Tools (C++)* + WebView2. Si `link.exe` resuelve al de Git (`link: extra operand`), falta MSVC o esta mal el PATH.
- Desde Git Bash los heredocs largos con comillas pueden romperse; escribe archivos con la herramienta Write.
- Dev: `pnpm tauri dev` · Build: `pnpm tauri build` · Typecheck: `pnpm typecheck` · Rust: `cd src-tauri && cargo check`.

## Arquitectura
Datos en `%LOCALAPPDATA%\Laralite`:
```
bin/<tool>/<version-id>/   binarios extraidos (zip aplanado si trae una carpeta raiz)
current/<tool>             junction a la version activa  <- esto es lo que va al PATH
cache/catalog.json         catalogo de versiones (una consulta por herramienta)
cache/downloads/           descargas parciales (.part)
logs/ data/<tool>/<id>/   logs de servicios y datos de BD (por version)
state.json                 { active: { tool: versionId } }
```
Cambiar de version = repuntar la junction; el PATH nunca cambia.

### Backend `src-tauri/src`
| Modulo | Responsabilidad |
|---|---|
| `tools.rs` | enum `Tool` + `meta()` (id, subcarpeta de PATH, archivo "probe", referer) |
| `paths.rs` | todas las rutas (`root`, `version_dir`, `current_link`, `path_entry`...) |
| `http.rs` | cliente reqwest unico, `get/text/json` (Referer opcional) |
| `catalog.rs` | proveedores por herramienta + cache en disco (`get(tool, force)`) |
| `install.rs` | descarga con progreso (evento `install-progress`), sha256, extraccion, activar, desinstalar |
| `fsx.rs` | json read/write, junctions, extract_zip |
| `env.rs` | variables y PATH de usuario en el registro + `WM_SETTINGCHANGE` |
| `php_ext.rs` | extensiones PHP: edita `php.ini` (`extension=`/`zend_extension=`) |
| `services.rs` | iniciar/detener Apache, MySQL, MariaDB, PostgreSQL (initdb + postgres.exe, stop con pg_ctl): procesos hijos sin ventana, log en `logs/<tool>.log`, datadir por version en `data/<tool>/<id>`, `httpd.conf` con `SRVROOT` reescrito, se detienen al cerrar la app |
| `phpmyadmin.rs` | `configure(dir)`: escribe `config.inc.php` (auth `config`, root sin clave => entra sin login); se llama desde `install.rs`. Corre como servicio con `php -S 127.0.0.1:8081` (PHP activo) en `services.rs` |
| `heidisql.rs` | HeidiSQL portable: `configure` escribe `portable_settings.txt` con conexiones locales (MySQL root, PostgreSQL postgres, sin clave); `launch` abre el exe activo (comando `launch_heidisql`) y repone lo que falte. Formato `clave<|||>tipo<|||>valor`: tipo 1=texto, 3=entero; `Port` va como texto; NetType 0=MySQL, 10=PostgreSQL; `Language=es` |
| `commands.rs` | comandos `#[tauri::command]`; registrar en `main.rs` |

Fuentes: PHP → `downloads.php.net/~windows/releases/releases.json` + listado `archives/`; Composer → `getcomposer.org/versions`; Apache → scraping de apachelounge.com (necesita Referer); MariaDB → `downloads.mariadb.org/rest-api`; PostgreSQL → HEAD a `get.enterprisedb.com` sobre rangos de versiones (ampliar en `catalog.rs::postgres`); HeidiSQL → GitHub releases (asset `*_64_Portable.zip`); phpMyAdmin → scraping de `phpmyadmin.net/files/` (>=5.x) y zip `files.phpmyadmin.net/.../all-languages.zip`; MySQL → sin API: HEAD a `cdn.mysql.com` sobre rangos de versiones conocidas (ampliar rangos en `catalog.rs::mysql`).

### Frontend `src`
- `lib/api.ts` (tipos + wrappers `invoke`), `lib/queries.ts` (`useOverview`, `useReleases`, `useAction`, `keys`), `lib/stores.ts` (progreso de descargas, toasts), `lib/tools.ts` (etiquetas), `lib/format.ts`.
- `features/` una pagina por seccion (Binarios, Extensiones PHP, Entorno); `components/` piezas compartidas (`ReleaseRow`, `AddForm`, `Toasts`).

## Como extender
Ver `docs/ai/extending.md` (nueva herramienta, nuevo comando, nueva pagina).

## Pendiente / ideas
Integrar PHP con Apache (modulo TS), DocumentRoot/vhosts + hosts, PECL/DLL extra para PHP, selector de carpeta raiz.
