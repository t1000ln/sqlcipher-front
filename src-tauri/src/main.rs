#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]


use std::ops::Deref;
use std::path::PathBuf;

use once_cell::sync::Lazy;

pub mod api;

pub static mut CONFIG_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| None);

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .setup(|app| {
            let cache_dir = app.path_resolver().app_cache_dir();
            println!("cache dir: {:?}", cache_dir);

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
        CONFIG_DIR.deref().unwrap().clone()
    }
}
