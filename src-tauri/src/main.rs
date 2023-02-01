#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]


#[macro_use]
extern crate rbatis;
extern crate rbdc;

use std::ops::Deref;
use std::path::PathBuf;

use once_cell::sync::Lazy;

use api::*;

pub mod api;
pub mod support;

pub static mut CONFIG_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| None);


fn main() {
    fast_log::init(fast_log::Config::new().console()).expect("log init fail");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_history,add_history,open_db,fetch_table_data,exec_custom_sql,update_table_data,remove_history_entry,get_table_sql,
            save_temp_notes,load_temp_notes
        ])
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
