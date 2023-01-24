//! 请描述文件用途。

use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use rbatis::Rbatis;
use rbdc::db::ConnectOptions;
use rbdc_sqlite::driver::SqliteDriver;
use rbdc_sqlite::SqliteConnectOptions;
use serde::{Deserialize, Serialize};

static OPENED_DBS: Lazy<Mutex<HashMap<String, Arc<Rbatis>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqliteMeta {
    obj_type: Option<String>,
    name: Option<String>,
    tbl_name: Option<String>,
    rootpage: Option<i64>,
    sql: Option<String>,
}

impl_select!(SqliteMeta{select_cols(table_column: &str) => "`order by name"}, "sqlite_master");

/// 打开或重新获取SQLITE数据库连接。
///
/// # Arguments
///
/// * `db_path`: 数据库文件路径。
/// * `key`: 可选的密钥字符串。
///
/// returns: Result<Arc<Rbatis>, Box<dyn Error, Global>>
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use rbatis::Rbatis;
/// let conn_ref: Arc<Rbatis> = open_db_connections(&"/home/foo/tmp/sqlite/sms.db".to_string(), &Some("123456".to_string())).unwrap();
/// let mut rb: &Rbatis = conn_ref.deref();
/// rb.exec("create table if not exists my_table (id text, name text, age integer)", vec![]).await.unwrap();
/// ```
pub fn open_db_connections(db_path: &String, key: &Option<String>) -> Result<Arc<Rbatis>, Box<dyn Error>> {
    let map_key = db_path.clone();
    let mut map = OPENED_DBS.lock()?;
    if !map.contains_key(&map_key) {
        let mut opts = SqliteConnectOptions::new();
        opts.set_uri(map_key.as_str()).unwrap();
        if let Some(key) = key {
            opts = opts.pragma("key", key.clone());
        }
        let rb = Rbatis::new();
        rb.init_opt(SqliteDriver {}, opts)?;
        map.insert(map_key.clone(), Arc::new(rb));
    }
    Ok(map.get(&map_key).unwrap().clone())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetaResult {
    db_path: String,
    table_names: Option<Vec<String>>,
    view_names: Option<Vec<String>>,
    key: Option<String>,
}

pub async fn load_tables(db_path: String, key: Option<String>) -> Result<MetaResult, Box<dyn Error>> {
    let rb = open_db_connections(&db_path, &key)?;
    let mut rb = rb.deref();

    let mut metas = SqliteMeta::select_cols(&mut rb, "type as obj_type,name").await?;
    let talbe_opt = Some(String::from("table"));
    let view_opt = Some(String::from("view"));
    let table_names: Vec<String> = metas.iter_mut().filter(|m| talbe_opt.eq(&m.obj_type)).map(|m| {
        return m.name.replace("".to_string()).unwrap();
    }).collect();
    let view_names: Vec<String> = metas.iter_mut().filter(|m| view_opt.eq(&m.obj_type)).map(|m| {
        return m.name.replace("".to_string()).unwrap();
    }).collect();

    let mut result = MetaResult { db_path: db_path, table_names: None, view_names: None, key };
    if !table_names.is_empty() {
        result.table_names = Some(table_names);
    }
    if !view_names.is_empty() {
        result.view_names = Some(view_names);
    }
    Ok(result)
}

pub async fn fetch_rows(db_path: String, table_name: String, limit: u64, key: Option<String>) {
    let mut rb = open_db_connections(&db_path, &key).unwrap();
    // TODO 待添加查询动态表结构数据
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    pub async fn test_open_db() {
        fast_log::init(fast_log::Config::new().console()).expect("log init fail");
        for i in 1..3 {
            let result = open_db_connections(&"/home/liuning/tmp/my.db/sms.db".to_string(), &Some("123456".to_string()));
            match result {
                Err(e) => assert!(false, "打开数据库失败 {}", e),
                Ok(rb) => {
                    println!("第 {} 次正常开启数据库 {:?}", i, rb);
                    let mut rb = rb.deref();
                    rb.exec("create table if not exists my_table (id text, name text, age integer)", vec![]).await.unwrap();
                    rb.exec("create view if not exists v_my_table as select id,name from my_table", vec![]).await.unwrap();

                    let metas = SqliteMeta::select_cols(&mut rb, "type as obj_type,name").await;
                    match metas {
                        Err(e) => assert!(false, "查询元数据出错 {}", e),
                        Ok(rows) => {
                            println!("第 {} 次查询到结果 {:?}", i, rows);
                        }
                    }
                }
            }
        }

        for i in 1..3 {
            let result = open_db_connections(&"/home/liuning/tmp/sqlite/my.db".to_string(), &Some("123456".to_string()));
            match result {
                Err(e) => assert!(false, "打开数据库失败 {}", e),
                Ok(rb) => {
                    println!("第 {} 次正常开启数据库 {:?}", i, rb);

                    // rb.exec("create table if not exists my_table (id text, name text, age integer)", vec![]).await.unwrap();
                    // rb.exec("create view if not exists v_my_table as select id,name from my_table", vec![]).await.unwrap();
                    let mut rb = rb.deref();
                    let metas = SqliteMeta::select_cols(&mut rb, "type as obj_type,name").await;
                    match metas {
                        Err(e) => assert!(false, "查询元数据出错 {}", e),
                        Ok(rows) => {
                            println!("第 {} 次查询到结果 {:?}", i, rows);
                        }
                    }
                }
            }
        }
    }

    #[tokio::test]
    pub async fn test_load_tables() {
        fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
        let load_result = load_tables("/home/liuning/tmp/my.db/sms.db".to_string(), Some("123456".to_string())).await;
        match load_result {
            Err(e) => assert!(false, "查询表名失败 {}", e),
            Ok(metas) => {
                println!("查询到表名列表 {:?}", metas);
            }
        }
    }
}
