//! 请描述文件用途。
use std::path::PathBuf;
use std::result;

use api_resp::ApiResp;
use serde_json::json;

use support::history;
use support::history::{add_open_history, get_open_history};

use crate::get_config_dir;

#[tauri::command]
pub async fn load_history(path: Option<String>) -> String {
    let data = get_open_history(if path.is_none() { get_config_dir() } else { PathBuf::from(path.unwrap()) });
    if data.is_some() {
        ApiResp::success(json!(data.unwrap())).to_json()
    } else {
        ApiResp::suc().to_json()
    }
}

#[tauri::command]
pub async fn add_history(path: String, cache_file: Option<String>) -> String {
    let new_path = PathBuf::from(&path);
    if new_path.exists() && new_path.is_file() {
        let name = new_path.file_name().unwrap().to_str().unwrap().to_string();
        let data_path = if cache_file.is_none() { get_config_dir() } else { PathBuf::from(cache_file.unwrap()) };
        let add_result = add_open_history(data_path, name, path);
        if let Err(e) = add_result {
            ApiResp::error(-1, format!("缓存时出错！{:?}", e)).to_json()
        } else {
            ApiResp::suc().to_json()
        }
    } else {
        ApiResp::error(-1, "目标文件不存在".to_string()).to_json()
    }
}
