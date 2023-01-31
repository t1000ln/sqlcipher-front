//! 请描述文件用途。
use std::path::PathBuf;

use api_resp::{ApiResp, TransformResult};
use log::error;
use serde_json::json;

use crate::get_config_dir;
use crate::support::history::{add_open_history, get_open_history, remove_open_history};
use crate::support::load_db::{edit_data, exec_sql, fetch_rows, fetch_table_sql, load_tables, remove_db_connection};

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
    let name = new_path.file_name().unwrap().to_str().unwrap().to_string();
    let data_path = if cache_file.is_none() { get_config_dir() } else { PathBuf::from(cache_file.unwrap()) };
    let add_result = add_open_history(data_path, name, path, key);
    if let Err(e) = add_result {
        error!("缓存时出错 {:?}", e);
        ApiResp::error(-1, e.to_string()).to_json()
    } else {
        ApiResp::suc().to_json()
    }
}

#[tauri::command]
pub async fn remove_history_entry(index: usize, cache_file: Option<String>) -> String {
    let data_path = if cache_file.is_none() { get_config_dir() } else { PathBuf::from(cache_file.unwrap()) };
    let remove_file_cache_result = remove_open_history(data_path, index);
    match remove_file_cache_result {
        Ok(op) => {
            if let Some(path) = op {
                let remove_db_pool_result = remove_db_connection(&path);
                if let Err(e) = remove_db_pool_result {
                    error!("移除数据库连接池时出错 {:?}", e);
                    return ApiResp::error(-1, e.to_string()).to_json();
                }
            }
            ApiResp::suc().to_json()
        }
        Err(e) => {
            error!("移除缓存时出错 {:?}", e);
            ApiResp::error(-1, e.to_string()).to_json()
        }
    }
}

#[tauri::command]
pub async fn open_db(db_path: String, key: Option<String>) -> String {
    let load_result = load_tables(db_path, key).await;
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
pub async fn fetch_table_data(db_path: String, table_name: String, limit: u64, key: Option<String>) -> String {
    fetch_rows(db_path, table_name.clone(), limit, key).await.to_json_str(format!("加载表 {} 的数据时出错", table_name))
}

#[tauri::command]
pub async fn exec_custom_sql(db_path: String, sql: String, key: Option<String>) -> String {
    exec_sql(db_path, sql.as_str(), key).await.to_json_str(format!("执行自定义SQL: {} 时出错", sql))
}

#[tauri::command]
pub async fn update_table_data(db_path: String, table_name: String, key: Option<String>, del_rows: Option<Vec<String>>, new_rows: Option<serde_json::Value>, edit_rows: Option<serde_json::Value>) -> String {
    edit_data(db_path, table_name, key, new_rows, edit_rows, del_rows).await.to_json_str("更新数据时出错")
}

#[tauri::command]
pub async fn get_table_sql(db_path: String, table_name: String, key: Option<String>) -> String {
    fetch_table_sql(db_path, key, table_name).await.to_json_str("查询目标SQL语句时出错")
}
