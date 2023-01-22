#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]


use std::ops::Deref;
use std::path::PathBuf;

use once_cell::sync::Lazy;

use api::*;

pub mod api;

pub static mut CONFIG_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| None);


fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_history,add_history])
        .setup(|app| {
            let cache_dir = app.path_resolver().app_cache_dir();
            if let Some(cache_dir) = cache_dir {
                if !cache_dir.exists() {
                    std::fs::create_dir_all(cache_dir.clone()).unwrap();
                }
                unsafe {
                    CONFIG_DIR.replace(cache_dir);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn get_config_dir() -> PathBuf {
    unsafe {
        CONFIG_DIR.deref().clone().unwrap()
    }
}
