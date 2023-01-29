//! 该模块为一些带有UI界面的应用程序，提供加载文件历史记录的接口方法。
//! 例如经常使用的"文件菜单->最近"这个菜单菜单项就可以使用该模块提供的接口方法。

use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use lazy_regex::regex_replace_all;
use log::error;
use serde::{Deserialize, Serialize};
use toml::de::Error as TomlError;
use toml::Value;
use toml::value::Table;

#[derive(Serialize, Deserialize)]
struct HisList {
    his: Vec<His>,
}

#[derive(Serialize, Deserialize)]
struct His {
    name: String,
    path: String,
    key: Option<String>,
}

/// 读取加载文件的历史列表。
///
/// # Arguments
///
/// * `data_path`: 历史记录保存目录。
///
/// returns: String 返回JSON格式的字符串。
///
/// # Examples
///
/// ```
/// let data_path = PathBuf::from("/home/john/tmp");
/// let content = get_open_history(data_path);
/// println!("content: {}", content);
/// ```
pub fn get_open_history(mut data_path: PathBuf) -> Option<Value> {
    data_path.push("history.toml");
    return if data_path.exists() {
        let mut file = File::open(data_path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();

        let his_list: Result<Table, TomlError> = toml::from_str(content.as_str());

        match his_list {
            Ok(his_list) => {
                let his_arr = his_list.get("His").unwrap();
                Some(his_arr.clone())
            }
            Err(e) => {
                error!("get open history error: {:?}", e);
                None
            }
        }
    } else {
        None
    };
}

#[derive(Debug)]
pub struct NotSupportedOs;

impl Display for NotSupportedOs {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { todo!() }
}

impl Error for NotSupportedOs {}

/// 新增加载文件的历史记录。
///
/// # Arguments
///
/// * `data_path`: 历史记录保存目录。
/// * `name`: 文件名。
/// * `path`: 文件路径。
///
/// returns: Result<(), Error> 返回执行成败信息。
///
/// # Examples
///
/// ```
/// let data_path = PathBuf::from("/home/john/tmp");
/// let result = add_open_history(data_path, "my.db".to_string(), "/home/john/.cache/my.db".to_string());
/// if let Err(e) = result {
///     assert!(false, "新增失败 {}", e);
/// }
/// ```
pub fn add_open_history(mut data_path: PathBuf, name: String, path: String, key: Option<String>) -> Result<(), Box<dyn Error>> {
    data_path.push("history.toml");
    let mut content = String::new();
    if data_path.exists() {
        let mut file = File::open(data_path.clone())?;
        file.read_to_string(&mut content)?;
    }

    let his_list: Result<Table, TomlError> = toml::from_str(content.as_str());
    match his_list {
        Ok(mut his_list) => {
            let list: Option<&mut Value> = his_list.get_mut("His");
            let new_content;
            match list {
                Some(list) => {
                    let mut new_entry = toml::map::Map::new();
                    new_entry.insert("name".to_string(), Value::String(name));
                    new_entry.insert("path".to_string(), Value::String(path));
                    if key.is_some() {
                        new_entry.insert("key".to_string(), Value::String(key.unwrap()));
                    }

                    let array = list.as_array_mut().unwrap();
                    array.insert(0, Value::Table(new_entry));

                    if array.len() > 20 {
                        array.remove(21);
                    }
                    new_content = toml::to_string(&his_list).unwrap();
                }
                None => {
                    if key.is_some() {
                        new_content = format!("[[His]]\nname = \"{}\"\npath = \"{}\"\nkey=\"{}\"\n\n", name, path, key.unwrap());
                    } else {
                        new_content = format!("[[His]]\nname = \"{}\"\npath = \"{}\"\n\n", name, path);
                    }
                }
            }

            write_content_to_file(data_path.clone(), &new_content)?;
            Ok(())
        }
        Err(e) => {
            error!("add open history error: {:?}", e);
            Err(Box::new(e))
        }
    }
}

fn write_content_to_file(data_path: PathBuf, content: &String) -> Result<(), Box<dyn Error>> {
    let processed = if cfg!(windows) {
        regex_replace_all!(r#"(?P<sep>\\+)"#, content, |_, _sep| "\\\\").to_string()
    } else { content.to_string() };
    let mut file = File::options().write(true).truncate(true).create(true).open(data_path)?;
    file.write(processed.as_bytes())?;
    file.flush()?;
    Ok(())
}


/// 清空所有的历史记录。
///
/// # Arguments
///
/// * `data_path`: 历史记录保存目录。
///
/// returns: Result<(), Error> 操作成败信息。
///
/// # Examples
///
/// ```
/// let data_path = PathBuf::from("/home/john/tmp");
/// let result = empty_open_history(data_path);
/// if let Err(e) = result {
///     assert!(false, "清空历史记录失败 {}", e);
/// }
/// ```
pub fn empty_open_history(mut data_path: PathBuf) -> std::io::Result<()> {
    data_path.push("history.toml");
    if data_path.exists() {
        fs::remove_file(data_path)?;
        Ok(())
    } else {
        Ok(())
    }
}

/// 移除某一条历史记录。
///
/// # Arguments
///
/// * `data_path`: 历史记录保存目录。
/// * `index`: 将要移除的历史记录索引编号，从0开始。
///
/// returns: Result<(), Error> 操作成败信息。
///
/// # Examples
///
/// ```
/// let data_path = PathBuf::from("/home/john/tmp");
/// let result = remove_open_history(data_path, 1);
/// if let Err(e) = result {
///     assert!(false, "移除历史记录失败 {}", e);
/// }
/// ```
pub fn remove_open_history(mut data_path: PathBuf, index: usize) -> Result<Option<String>, Box<dyn Error>> {
    data_path.push("history.toml");
    let mut content = String::new();
    if data_path.exists() {
        let mut file = File::open(data_path.clone())?;
        file.read_to_string(&mut content)?;
    }

    let his_list: Result<Table, TomlError> = toml::from_str(content.as_str());
    match his_list {
        Ok(mut his_list) => {
            let mut removed_path: Option<String> = None;
            let list: Option<&mut Value> = his_list.get_mut("His");
            match list {
                Some(list) => {
                    let array = list.as_array_mut().unwrap();
                    if index < array.len() {
                        let entry = array.remove(index);
                        let entry_path: Option<&Value> = entry.as_table().unwrap().get("path");
                        if let Some(path) = entry_path {
                            removed_path = Some(path.as_str().unwrap().to_string());
                        }
                    }

                    let new_content = toml::to_string(&his_list).unwrap();

                    // 从文件头部开始写入内容。
                    write_content_to_file(data_path.clone(), &new_content)?;
                }
                None => {}
            }

            Ok(removed_path)
        }
        Err(e) => {
            error!("add open history error: {:?}", e);
            Err(Box::new(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_get_open_history() {
        let data_path = env::temp_dir();
        let content = get_open_history(data_path);
        println!("content: {:?}", content);
    }

    #[test]
    fn test_add_open_history() {
        let data_path = env::temp_dir();
        let path = if cfg!(windows) {
            env::temp_dir().as_os_str().to_str().unwrap().to_string()
        } else { "/home/liuning/.cache/my.db".to_string() };
        println!("path: {:?}", path);
        let result = add_open_history(data_path, "my3.db".to_string(), path, Some("123456".to_string()));
        if let Err(e) = result {
            assert!(false, "新增失败 {}", e);
        }
    }

    #[test]
    fn test_clear_open_history() {
        let data_path = env::temp_dir();
        let result = empty_open_history(data_path);
        if let Err(e) = result {
            assert!(false, "清空历史记录失败 {}", e);
        }
    }

    #[test]
    fn test_remove_open_history() {
        let data_path = env::temp_dir();
        let result = remove_open_history(data_path, 1);
        match result {
            Err(e) => { assert!(false, "移除历史记录失败 {}", e); }
            Ok(ov) => {
                println!("移除的条目内容为：{:?}", ov);
            }
        }
    }

    #[test]
    fn test_regex() {
        let content = "C:\\Users\\\\Foo\\bar\\";
        println!("original content: {}", content);
        let replaced = regex_replace_all!(r#"(?P<g>\\+)"#, content, |_, g| {
            println!("{}", g);
            "\\\\"
        });
        println!("replaced: {}", replaced);
    }
}
