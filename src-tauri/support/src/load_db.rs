//! 请描述文件用途。

use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use api_resp::{ApiResp, DaoResult};
use once_cell::sync::Lazy;
use rbatis::Rbatis;
use rbdc::db::ConnectOptions;
use rbdc_sqlite::driver::SqliteDriver;
use rbdc_sqlite::SqliteConnectOptions;
use rbs::{to_value, Value};
use regex::Regex;
use serde::{Deserialize, Serialize};

static OPENED_DBS: Lazy<Mutex<HashMap<String, Arc<Rbatis>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
static COLUMN_NAME_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"\((.+)\)").unwrap());

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

/// 加载已定义的表和视图列表。
///
/// # Arguments
///
/// * `db_path`: 数据库文件路径。
/// * `key`: 可选的密钥。
///
/// returns: Result<MetaResult, Box<dyn Error, Global>>
///
/// # Examples
///
/// ```
/// fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
/// let load_result = load_tables("/home/liuning/tmp/my.db/sms.db".to_string(), Some("123456".to_string())).await;
/// match load_result {
///     Err(e) => assert!(false, "查询表名失败 {}", e),
///     Ok(metas) => {
///         println!("查询到表名列表 {:?}", metas);
///     }
/// }
/// ```
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

#[derive(Serialize, Deserialize)]
pub struct TableData {
    cols: Vec<String>,
    rows: Vec<HashMap<String, Value>>,
}

/// 无条件查询目标表的数据（仅限制返回条数）。
///
/// # Arguments
///
/// * `db_path`: 数据库文件路径。
/// * `table_name`: 目标表名。
/// * `limit`: 限制返回条数，应大于0。
/// * `key`: 可选的密钥。
///
/// returns: Result<ApiResp, Box<dyn Error, Global>>
///
/// # Examples
///
/// ```
/// fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
/// let result = fetch_rows("/home/liuning/tmp/sqlite/my.db".to_string(), "my_table".to_string(), 10, Some("123456".to_string())).await;
/// if let Err(e) = result {
///     assert!(false, "查询数据库失败 {}", e);
/// } else {
///     println!("查询到数据 {:?}", result.to_json_str("查询出错"));
/// }
/// ```
pub async fn fetch_rows(db_path: String, table_name: String, limit: u64, key: Option<String>) -> DaoResult {
    let conn = open_db_connections(&db_path, &key)?;
    let rb = conn.deref();

    // 获取目标表的字段名列表
    let meta: String = rb.fetch_decode("select sql from sqlite_master where name = ?", vec![to_value!(&table_name)]).await?;
    let mut cols: Vec<String> = vec![];
    let re = COLUMN_NAME_REG.captures(&meta);
    if let Some(cap) = re {
        let c = cap.get(1);
        if let Some(c) = c {
            let sc = c.as_str().split(",");
            for s in sc {
                for part in s.split(char::is_whitespace) {
                    if !"".eq(part) {
                        cols.push(part.to_string());
                        break;
                    }
                }
            }
        } else {
            return Ok(ApiResp::error(-1, "检查表结构时出错".to_string()));
        }
    }

    // 查询数据
    let rows: Vec<HashMap<String, Value>> = rb.fetch_decode(format!("select rowid,* from {} limit ?", table_name).as_str(), vec![to_value!(limit)]).await?;
    Ok(ApiResp::success(serde_json::json!(TableData { cols, rows })))
}


/// 执行用户输入的SQL语句。
///
/// # Arguments
///
/// * `db_path`: 数据库文件路径。
/// * `sql`: 用户SQL。
/// * `key`: 可选的密钥。
///
/// returns: Result<ApiResp, Box<dyn Error, Global>> 若是查询语句(或explain语句)返回`Vec<HashMap<String, Value>>`结构的数据，否则返回更新行数信息。
///
/// # Examples
///
/// ```
/// fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
/// let query_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "select rowid, * from my_table", Some("123456".to_string())).await;
/// if let Err(e) = query_result {
///     assert!(false, "查询数据库失败: {}", e);
/// } else {
///     println!("查询到数据 {:?}", query_result.to_json_str("查询出错"));
/// }
///
/// let insert_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "insert into my_table(id,name) values(2,'lisi')", Some("123456".to_string())).await;
/// if let Err(e) = insert_result {
///     assert!(false, "更新数据库失败: {}", e);
/// } else {
///     println!("本次操作结果 {:?}", insert_result.to_json_str("操作出错"));
/// }
/// ```
pub async fn exec_sql(db_path: String, sql: &str, key: Option<String>) -> DaoResult {
    let conn = open_db_connections(&db_path, &key)?;
    let rb = conn.deref();

    match sql.trim().to_lowercase().as_str() {
        s if s.starts_with("select") => {
            let rows: Vec<HashMap<String, Value>> = rb.fetch_decode(s, vec![]).await?;
            Ok(ApiResp::success(serde_json::json!(rows)))
        }
        s if s.starts_with("explain") => {
            let result: Vec<HashMap<String, Value>> = rb.fetch_decode(s, vec![]).await?;
            Ok(ApiResp::success(serde_json::json!(result)))
        }
        _ => {
            let result = rb.exec(sql, vec![]).await?;
            Ok(ApiResp::success(serde_json::json!(result)))
        }
    }
}

pub async fn edit_data(db_path: String, table_name: String, key: Option<String>, new_rows: Vec<HashMap<String, String>>, update_rows: Vec<HashMap<String, String>>, del_rows: Vec<u64>) -> DaoResult {
    Ok(ApiResp::suc())
}


#[cfg(test)]
mod tests {
    use api_resp::TransformResult;

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

    #[tokio::test]
    pub async fn test_fetch_rows() {
        fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
        let result = fetch_rows("/home/liuning/tmp/sqlite/my.db".to_string(), "my_table".to_string(), 10, Some("123456".to_string())).await;
        if let Err(e) = result {
            assert!(false, "查询数据库失败 {}", e);
        } else {
            println!("查询到数据 {:?}", result.to_json_str("查询出错"));
        }
    }

    #[tokio::test]
    pub async fn test_exec_sql() {
        fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
        let query_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "select rowid, * from my_table", Some("123456".to_string())).await;
        if let Err(e) = query_result {
            assert!(false, "查询数据库失败: {}", e);
        } else {
            println!("查询到数据 {:?}", query_result.to_json_str("查询出错"));
        }

        let insert_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "insert into my_table(id,name) values(3,'lisi')", Some("123456".to_string())).await;
        if let Err(e) = insert_result {
            assert!(false, "更新数据库失败: {}", e);
        } else {
            println!("本次操作结果 {:?}", insert_result.to_json_str("操作出错"));
        }

        let delete_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "delete from my_table where rowid = 3", Some("123456".to_string())).await;
        if let Err(e) = delete_result {
            assert!(false, "更新数据库失败: {}", e);
        } else {
            println!("本次操作结果 {:?}", delete_result.to_json_str("操作出错"));
        }

        let create_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "create table if not exists my_table(id integer,etime text,name text)", Some("123456".to_string())).await;
        if let Err(e) = create_result {
            assert!(false, "更新数据库失败: {}", e);
        } else {
            println!("本次操作结果 {:?}", create_result.to_json_str("操作出错"));
        }

        let explain_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "explain query plan select rowid,* from my_table", Some("123456".to_string())).await;
        if let Err(e) = explain_result {
            assert!(false, "更新数据库失败: {}", e);
        } else {
            println!("本次操作结果 {:?}", explain_result.to_json_str("操作出错"));
        }
    }
}
