//! 请描述文件用途。
use std::collections::HashMap;
use std::error::Error;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use api_resp::{ApiResp, DaoResult, rollback};
use once_cell::sync::Lazy;
use rbatis::Rbatis;
use rbdc::db::ConnectOptions;
use rbdc_sqlite::driver::SqliteDriver;
use rbdc_sqlite::SqliteConnectOptions;
use rbs::{to_value, Value};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

static OPENED_DBS: Lazy<Mutex<HashMap<String, Arc<Rbatis>>>> = Lazy::new(|| Mutex::new(HashMap::new()));
// static COLUMN_NAME_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"\((.+)\)").unwrap());
// static CREATE_VIEW_REG: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(?i)CREATE\s+VIEW").unwrap());

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqliteMeta {
    obj_type: Option<String>,
    name: Option<String>,
    tbl_name: Option<String>,
    rootpage: Option<i64>,
    sql: Option<String>,
}

impl_select!(SqliteMeta{select_cols(table_column: &str) => "`order by name"}, "sqlite_master");


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TableInfo {
    data_type: String,
    name: String,
}

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

    /*
    使用ruqlite绑定的sqlipher包，自动创建加密库文件。
     */
    let conn = Connection::open(db_path).unwrap();
    if let Some(key) = key {
        conn.pragma_update(None, "key", key.clone()).unwrap();
    }

    let mut map = OPENED_DBS.lock()?;
    if !map.contains_key(&map_key) {
        let mut opts = SqliteConnectOptions::new();
        opts.set_uri(map_key.as_str()).unwrap();
        if let Some(key) = key {
            opts = opts.pragma("key", key.clone());
        }
        opts = opts.create_if_missing(false);
        let rb = Rbatis::new();
        rb.init_opt(SqliteDriver {}, opts)?;
        map.insert(map_key.clone(), Arc::new(rb));
    }
    Ok(map.get(&map_key).unwrap().clone())
}

pub fn remove_db_connection(db_path: &String) -> Result<(), Box<dyn Error>> {
    let mut map = OPENED_DBS.lock()?;
    map.remove(db_path);
    Ok(())
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
/// let load_result = load_tables("/home/foo/tmp/sqlite/sms.db".to_string(), Some("123456".to_string())).await;
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
    let table_opt = Some(String::from("table"));
    let view_opt = Some(String::from("view"));

    let mut names = |opt: Option<String>| -> Vec<String> {
        metas.iter_mut().filter(|m| opt.eq(&m.obj_type)).map(|m| {
            return m.name.replace("".to_string()).unwrap();
        }).collect()
    };
    let table_names = names(table_opt);
    let view_names = names(view_opt);

    let mut result = MetaResult { db_path, table_names: None, view_names: None, key };
    if !table_names.is_empty() {
        result.table_names = Some(table_names);
    }
    if !view_names.is_empty() {
        result.view_names = Some(view_names);
    }
    Ok(result)
}

pub async fn fetch_table_sql(db_path: String, key: Option<String>, table_name: String) -> DaoResult {
    let _rb = open_db_connections(&db_path, &key)?;
    let rb = _rb.deref();
    let sql: String = rb.fetch_decode("select sql from sqlite_master where name = ?", vec![to_value!(table_name)]).await?;
    Ok(ApiResp::success(serde_json::Value::String(sql)))
}


#[derive(Serialize, Deserialize)]
pub struct TableData {
    cols: Vec<TableInfo>,
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

    /*
    获取目标表或视图的字段名列表
     */
    let mut cols: Vec<TableInfo> = vec![];
    let table_info_sql = format!("pragma table_info({})", table_name);
    let result: Vec<HashMap<String, rbs::Value>> = rb.fetch_decode(table_info_sql.as_str(), vec![]).await.unwrap();
    for r in result {
        let mut data_type: String = String::new();
        let mut name: String = String::new();
        for (k, v) in r {
            if k.eq("name") {
                name.push_str(v.as_str().unwrap());
            } else if k.eq("type") {
                data_type.push_str(v.as_str().unwrap());
            }
        }
        cols.push(TableInfo { data_type, name });
    }

    /*
    检查目标是表还是视图。
     */
    let mut is_table = true;
    let target_type: String = rb.fetch_decode("select type from sqlite_master where name = ?", vec![to_value!(&table_name)]).await.unwrap();
    if !target_type.eq("table") {
        is_table = false;
    }

    /*
    查询数据，若目标是表则附加rowid字段，以便后续修改操作。
     */
    let sql = format!("select {} * from {} limit ?", if is_table { "rowid," } else { "" }, table_name);
    let rows: Vec<HashMap<String, Value>> = rb.fetch_decode(sql.as_str(), vec![to_value!(limit)]).await?;
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
        s if s.starts_with("select") || s.starts_with("pragma") || s.starts_with("explain") => {
            let rows: Vec<HashMap<String, Value>> = rb.fetch_decode(s, vec![]).await?;
            Ok(ApiResp::success(serde_json::json!(rows)))
        }
        _ => {
            let result = rb.exec(sql, vec![]).await?;
            Ok(ApiResp::success(serde_json::json!(result)))
        }
    }
}

/// 编辑目标表的数据，包括删除、更新和新增。
///
/// # Arguments
///
/// * `db_path`: 数据库文件路径。
/// * `table_name`: 目标表名。
/// * `key`: 可选的密钥。
/// * `new_rows`: 可选的新增数据。有效值是个`array`，其每个元素表示一行新增的数据，每行数据是`map`结构，`key:value`关系为`字段名:字段值`。
/// * `update_rows`: 可选的更新数据。有效值为双层`map`结构，第一层`map`的`key`为`rowid`，`value`为待更新的行字段数据，第二层`map`表示待更新的字段名和值。
/// * `del_rows`: 可选的删除数据。有效值为目标表的`rowid`数组。
///
/// returns: Result<ApiResp, Box<dyn Error, Global>> 返回操作结果。
///
/// # Examples
///
/// ```
/// use api_resp::TransformResult;
/// use serde_json::json;
///
/// use names::{Generator, Name};
/// use rand::prelude::*;
///
/// #[derive(Serialize)]
/// struct MyTable {
///     id: u64,
///     name: String,
///     etime: String,
/// }
///
/// fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
///
/// let mut rng = rand::thread_rng();
/// let mut generator = Generator::with_naming(Name::Numbered);
///
/// let mut del_rowids: Vec<String> = vec![];
/// for _ in 0..3 {
///     let num: u64 = rng.gen_range(1000..=2000);
///     del_rowids.push(num.to_string());
/// }
/// let del_rows = Some(del_rowids);
///
/// let mut nw: Vec<MyTable> = vec![];
/// for _ in 0..3 {
///     let mt = MyTable { id: rng.gen_range(1000..=2000), name: generator.next().unwrap(), etime: rng.gen_range(300000..=400000).to_string() };
///     nw.push(mt);
/// }
/// let new_rows = json!(nw);
/// // let new_rows: serde_json::Value = json!([
/// //     {"id": 30, "etime": null, "name": "zhangsan"},
/// //     {"id": 31, "etime": "3123512", "name": "wangwu"},
/// // ]);
///
/// let mut uw: HashMap<String, HashMap<String, String>> = HashMap::new();
/// for _ in 0..2 {
///     let rowid = rng.gen_range(1..=100).to_string();
///     let mut row: HashMap<String, String> = HashMap::new();
///     row.insert("etime".to_string(), rng.gen_range(300000..=400000).to_string());
///     row.insert("name".to_string(), generator.next().unwrap());
///     uw.insert(rowid, row);
/// }
/// let update_rows: serde_json::Value = json!(uw);
/// // let update_rows: serde_json::Value = json!({
/// //     "3": {"etime": "31235"},
/// //     "5": {"name": "new name"}
/// // });
///
/// let result = edit_data("/home/foo/tmp/sqlite/my.db".to_string(), "my_table".to_string(), Some("123456".to_string()), Some(new_rows), Some(update_rows), del_rows).await;
/// match result {
///     Ok(r) => {
///         if !r.is_success() {
///             assert!(false, "更新数据库失败 {}", r.get_message());
///         }
///     }
///     Err(e) => {
///         assert!(false, "更新数据库失败 {}", e);
///     }
/// }
/// ```
pub async fn edit_data(db_path: String, table_name: String, key: Option<String>, new_rows: Option<serde_json::Value>, update_rows: Option<serde_json::Value>, del_rows: Option<Vec<String>>) -> DaoResult {
    let conn = open_db_connections(&db_path, &key)?;
    let rb = conn.deref();
    let mut tx = rb.acquire_begin().await?;

    /*
    删除数据
     */
    if let Some(del_rows) = del_rows {
        let rowids = del_rows.join(",");
        let sql = format!("delete from {} where rowid in ({})", table_name, rowids);
        let result = tx.exec(sql.as_str(), vec![]).await;
        rollback!(result, tx, -1);
    }

    /*
    更新数据
     */
    if let Some(update_rows) = update_rows {
        // 转化为map类型
        if let serde_json::Value::Object(rows_map) = update_rows {
            // 遍历每一行数据
            for (rowid, row) in rows_map {
                // 每一行更新数据是map类型的
                if let serde_json::Value::Object(col_map) = row {
                    let mut fields: Vec<String> = vec![];
                    let mut args: Vec<rbs::Value> = vec![];

                    /*
                    拼装待更新的字段语句，结果类似"id=?,name=?,age=?"；
                    同时也将json类型的数值转换为rbatis定义的数值类型。
                     */
                    for (col, new_val) in col_map {
                        fields.push(format!("{}=?", col));
                        args.push(rbs::to_value!(new_val));
                    }
                    let fields_part = fields.join(",");

                    let sql = format!("update {} set {} where rowid = {}", table_name, fields_part, rowid);
                    let result = tx.exec(sql.as_str(), args).await;
                    rollback!(result, tx, -1);
                }
            }
        }
    }

    /*
    新增数据
     */
    if let Some(new_rows) = new_rows {
        if let serde_json::Value::Array(rows) = new_rows {
            for row in rows {
                if let serde_json::Value::Object(row_map) = row {
                    let mut args: Vec<rbs::Value> = vec![];
                    let mut cols: Vec<String> = vec![];
                    let mut params: Vec<String> = vec![];

                    for (col, val) in row_map {
                        cols.push(col);
                        args.push(rbs::to_value!(val));
                        params.push("?".to_string());
                    }

                    let sql = format!("insert into {} ({}) values ({})", table_name, cols.join(","), params.join(","));

                    let result = tx.exec(sql.as_str(), args).await;
                    rollback!(result, tx, -1);
                }
            }
        }
    }

    let cr = tx.commit().await?;
    if cr {
        Ok(ApiResp::suc())
    } else {
        Ok(ApiResp::error(-1, "提交更新事务失败".to_string()))
    }
}


#[cfg(test)]
mod tests {
    use api_resp::TransformResult;
    use names::{Generator, Name};
    use rand::prelude::*;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    pub async fn test_open_db() {
        fast_log::init(fast_log::Config::new().console()).expect("log init fail");
        let db_path = "/home/liuning/tmp/sqlite/my.db".to_string();
        let mut key = Some("123456".to_string());
        for i in 1..3 {
            let result = open_db_connections(&db_path, &key);
            match result {
                Err(e) => assert!(false, "打开数据库失败 {}", e),
                Ok(rb) => {
                    println!("第 {} 次正常开启数据库 {:?}", i, rb);
                    let mut rb = rb.deref();
                    rb.exec("create table if not exists my_table (id text, name text, etime integer)", vec![]).await.unwrap();
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

        key = None;
        for i in 1..3 {
            let result = open_db_connections(&db_path, &key);
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
        let load_result = load_tables("/home/liuning/tmp/sqlite/my.db".to_string(), Some("123456".to_string())).await;
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
        let result = fetch_rows("/home/liuning/tmp/sqlite/my.db".to_string(), "v_my_table".to_string(), 10, Some("123456".to_string())).await;
        if let Err(e) = result {
            assert!(false, "查询数据库失败 {}", e);
        } else {
            println!("查询到数据 {:?}", result.to_json_str("查询出错"));
        }
    }

    #[tokio::test]
    pub async fn test_exec_sql() {
        fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
        // let query_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "select rowid, * from my_table", Some("123456".to_string())).await;
        let query_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "pragma table_info(my_table)", Some("123456".to_string())).await;
        if let Err(e) = query_result {
            assert!(false, "查询数据库失败: {}", e);
        } else {
            println!("查询到数据 {:?}", query_result.to_json_str("查询出错"));
        }

        // let insert_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "insert into my_table(id,name) values(3,'lisi')", Some("123456".to_string())).await;
        // if let Err(e) = insert_result {
        //     assert!(false, "更新数据库失败: {}", e);
        // } else {
        //     println!("本次操作结果 {:?}", insert_result.to_json_str("操作出错"));
        // }
        //
        // let delete_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "delete from my_table where rowid = 3", Some("123456".to_string())).await;
        // if let Err(e) = delete_result {
        //     assert!(false, "更新数据库失败: {}", e);
        // } else {
        //     println!("本次操作结果 {:?}", delete_result.to_json_str("操作出错"));
        // }
        //
        // let create_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "create table if not exists my_table(id integer,etime text,name text)", Some("123456".to_string())).await;
        // if let Err(e) = create_result {
        //     assert!(false, "更新数据库失败: {}", e);
        // } else {
        //     println!("本次操作结果 {:?}", create_result.to_json_str("操作出错"));
        // }
        //
        // let explain_result = exec_sql("/home/liuning/tmp/sqlite/my.db".to_string(), "explain query plan select rowid,* from my_table", Some("123456".to_string())).await;
        // if let Err(e) = explain_result {
        //     assert!(false, "更新数据库失败: {}", e);
        // } else {
        //     println!("本次操作结果 {:?}", explain_result.to_json_str("操作出错"));
        // }
    }

    #[derive(Serialize)]
    struct MyTable {
        id: u64,
        name: String,
        etime: String,
    }

    #[tokio::test]
    pub async fn test_edit_data() {
        fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");

        let mut rng = rand::thread_rng();
        let mut generator = Generator::with_naming(Name::Numbered);

        let mut del_rowids: Vec<String> = vec![];
        for _ in 0..3 {
            let num: u64 = rng.gen_range(1000..=2000);
            del_rowids.push(num.to_string());
        }
        let del_rows = Some(del_rowids);

        let mut nw: Vec<MyTable> = vec![];
        for _ in 0..3 {
            let mt = MyTable { id: rng.gen_range(1000..=2000), name: generator.next().unwrap(), etime: rng.gen_range(300000..=400000).to_string() };
            nw.push(mt);
        }
        let new_rows = json!(nw);

        let mut uw: HashMap<String, HashMap<String, String>> = HashMap::new();
        for _ in 0..2 {
            let rowid = rng.gen_range(1..=100).to_string();
            let mut row: HashMap<String, String> = HashMap::new();
            row.insert("etime".to_string(), rng.gen_range(300000..=400000).to_string());
            row.insert("name".to_string(), generator.next().unwrap());
            uw.insert(rowid, row);
        }
        let update_rows: serde_json::Value = json!(uw);

        let result = edit_data("/home/liuning/tmp/sqlite/my.db".to_string(), "my_table".to_string(), Some("123456".to_string()), Some(new_rows), Some(update_rows), del_rows).await;
        match result {
            Ok(r) => {
                if !r.is_success() {
                    assert!(false, "更新数据库失败 {}", r.get_message());
                }
            }
            Err(e) => {
                assert!(false, "更新数据库失败 {}", e);
            }
        }
    }

    #[tokio::test]
    pub async fn test_remove_db_connection() {
        fast_log::init(fast_log::Config::new().console()).expect("log init fail");
        let db_path = "/home/liuning/tmp/sqlite/my.db".to_string();

        /*
        先创建一个连接池
         */
        let key = Some("123456".to_string());
        for i in 1..3 {
            let result = open_db_connections(&db_path, &key);
            match result {
                Err(e) => assert!(false, "打开数据库失败 {}", e),
                Ok(rb) => {
                    println!("第 {} 次正常开启数据库 {:?}", i, rb);
                    let mut rb = rb.deref();
                    rb.exec("create table if not exists my_table (id text, name text, etime integer)", vec![]).await.unwrap();
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

        let result = remove_db_connection(&db_path);
        match result {
            Err(e) => assert!(false, "移除数据库连接池失败 {}", e),
            Ok(()) => {
                let map = OPENED_DBS.lock().unwrap();
                assert!(!map.contains_key(&db_path), "移除连接池失败");
            }
        }
    }

    #[tokio::test]
    pub async fn test_meta() {
        fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
        let db_path = "/home/liuning/tmp/sqlite/my.db".to_string();
        let key = Some("123456".to_string());
        let conn = open_db_connections(&db_path, &key).unwrap();
        let rb = conn.deref();

        let result: Vec<HashMap<String, rbs::Value>> = rb.fetch_decode("pragma table_info(v_my_table)", vec![]).await.unwrap();
        for r in result {
            for (k, v) in r {
                if k.eq("name") || k.eq("type") {
                    println!("{} = {}", k, v.as_str().unwrap());
                }
            }
        }
        assert!(true)
    }

    #[tokio::test]
    pub async fn test_fetch_table_sql() {
        fast_log::init(fast_log::Config::new().console()).expect("rbatis init fail");
        let db_path = "/home/liuning/tmp/sqlite/my.db".to_string();
        let key = Some("123456".to_string());
        let ret = fetch_table_sql(db_path, key, "my_table".to_string()).await.unwrap();
        println!("ret {:?}", ret);
    }
}
