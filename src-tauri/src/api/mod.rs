//! 请描述文件用途。
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;

use api_resp::{ApiResp, TransformResult};
use log::error;
use serde_json::json;

use support::history::{add_open_history, get_open_history};
use support::load_db::{exec_sql, fetch_rows, load_tables};

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
pub async fn add_history(path: String, cache_file: Option<String>, key: Option<String>) -> String {
    let new_path = PathBuf::from(&path);
    if new_path.exists() && new_path.is_file() {
        let name = new_path.file_name().unwrap().to_str().unwrap().to_string();
        let data_path = if cache_file.is_none() { get_config_dir() } else { PathBuf::from(cache_file.unwrap()) };
        let add_result = add_open_history(data_path, name, path, key);
        if let Err(e) = add_result {
            error!("缓存时出错 {:?}", e);
            ApiResp::error(-1, e.to_string()).to_json()
        } else {
            ApiResp::suc().to_json()
        }
    } else {
        ApiResp::error(-1, "目标文件不存在".to_string()).to_json()
    }
}


#[tauri::command]
pub async fn open_db(db_path: String) -> String {
    let load_result = load_tables(db_path, Some("123456".to_string())).await;
    match load_result {
        Ok(metas) => {
            ApiResp::success(json!(metas)).to_json()
        }
        Err(e) => {
            error!("查询表名列表出错 {:?}", e);
            ApiResp::error(-1, e.to_string()).to_json()
        }
    }
}


#[tauri::command]
pub async fn fetch_table_data(db_path: String, table_name: String, limit: u64) -> String {
    fetch_rows(db_path, table_name.clone(), limit, Some("123456".to_string())).await.to_json_str(format!("加载表 {} 的数据时出错", table_name))
}

#[tauri::command]
pub async fn exec_custom_sql(db_path: String, sql: String, key: Option<String>) -> String {
    exec_sql(db_path, sql.as_str(), key).await.to_json_str(format!("执行自定义SQL: {} 时出错", sql))
}

#[tauri::command]
pub async fn update_table_data(db_path: String, table_name: String, key: Option<String>, del_rows: Option<Vec<u64>>, new_rows: Option<serde_json::Value>, edit_rows: Option<serde_json::Value>) -> String {
    println!("待删除的数据：{:?}", del_rows);
    println!("待添加的数据：{:?}", new_rows);
    println!("待更新的数据：{:?}", edit_rows);
    exit(0);
    ApiResp::suc().to_json()
}
