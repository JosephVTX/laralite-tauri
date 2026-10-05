#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod catalog;
mod commands;
mod env;
mod error;
mod fsx;
mod heidisql;
mod http;
mod install;
mod paths;
mod php_ext;
mod phpmyadmin;
mod services;
mod tools;

fn main() {
    let services = services::Services::default();
    tauri::Builder::default()
        .manage(services.clone())
        .invoke_handler(tauri::generate_handler![
            commands::overview,
            commands::get_releases,
            commands::install_release,
            commands::uninstall_version,
            commands::activate_version,
            commands::set_in_path,
            commands::env_vars,
            commands::env_set,
            commands::env_remove,
            commands::path_entries,
            commands::path_add,
            commands::path_remove,
            commands::php_extensions,
            commands::php_set_extension,
            commands::open_external,
            commands::launch_heidisql,
            commands::services_status,
            commands::service_start,
            commands::service_stop,
        ])
        .build(tauri::generate_context!())
        .expect("error al iniciar Laralite")
        .run(move |_, event| {
            if let tauri::RunEvent::Exit = event {
                services::stop_all(&services);
            }
        });
}
