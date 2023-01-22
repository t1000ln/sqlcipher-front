//! 该模块为一些带有UI界面的应用程序，提供加载文件历史记录的接口方法。
//! 例如经常使用的"文件菜单->最近"这个菜单菜单项就可以使用该模块提供的接口方法。

use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::FileExt;
use std::path::PathBuf;

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
pub fn add_open_history(mut data_path: PathBuf, name: String, path: String, key: Option<String>) -> std::io::Result<()> {
    data_path.push("history.toml");
    let mut file = File::options().write(true).read(true).create(true).append(false).open(data_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let his_list: Result<Table, TomlError> = toml::from_str(content.as_str());
    match his_list {
        Ok(mut his_list) => {
            let list: Option<&mut Value> = his_list.get_mut("His");
            match list {
                Some(list) => {
                    let mut new_entry = toml::map::Map::new();
                    new_entry.insert("name".to_string(), toml::Value::String(name));
                    new_entry.insert("path".to_string(), toml::Value::String(path));
                    if key.is_some() {
                        new_entry.insert("key".to_string(), toml::Value::String(key.unwrap()));
                    }

                    let array = list.as_array_mut().unwrap();
                    array.insert(0, toml::Value::Table(new_entry));

                    if array.len() > 20 {
                        array.remove(21);
                    }
                    let new_content = toml::to_string(&his_list).unwrap();
                    // 从文件头部开始写入内容。
                    file.write_at(new_content.as_bytes(), 0)?;
                }
                None => {
                    let new_content;
                    if key.is_some() {
                        new_content = format!("[[His]]\nname = \"{}\"\npath = \"{}\"\nkey=\"{}\"\n\n", name, path, key.unwrap());
                    } else {
                        new_content = format!("[[His]]\nname = \"{}\"\npath = \"{}\"\n\n", name, path);
                    }
                    file.write(new_content.as_bytes())?;
                }
            }

            file.flush()?;

            Ok(())
        }
        Err(e) => {
            error!("add open history error: {:?}", e);
            Err(e.into())
        }
    }
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
pub fn remove_open_history(mut data_path: PathBuf, index: usize) -> std::io::Result<()> {
    data_path.push("history.toml");
    let mut file = File::options().write(true).read(true).create(true).append(false).open(data_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let his_list: Result<Table, TomlError> = toml::from_str(content.as_str());
    match his_list {
        Ok(mut his_list) => {
            let list: Option<&mut Value> = his_list.get_mut("His");
            match list {
                Some(list) => {
                    let array = list.as_array_mut().unwrap();
                    if index < array.len() {
                        array.remove(index);
                    }

                    // 清空文件内容，并收缩容量到0长度。
                    file.set_len(0)?;

                    let new_content = toml::to_string(&his_list).unwrap();
                    // 从文件头部开始写入内容。
                    file.write_at(new_content.as_bytes(), 0)?;
                    file.flush()?;
                }
                None => {}
            }

            Ok(())
        }
        Err(e) => {
            error!("add open history error: {:?}", e);
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_get_open_history() {
        let data_path = PathBuf::from("/home/liuning/tmp");
        let content = get_open_history(data_path);
        println!("content: {:?}", content);
    }

    #[test]
    fn test_add_open_history() {
        let data_path = PathBuf::from("/home/liuning/tmp");
        let result = add_open_history(data_path, "my2.db".to_string(), "/home/liuning/.cache/my.db".to_string(), Some("123456".to_string()));
        if let Err(e) = result {
            assert!(false, "新增失败 {}", e);
        }
    }

    #[test]
    fn test_clear_open_history() {
        let data_path = PathBuf::from("/home/liuning/tmp");
        let result = empty_open_history(data_path);
        if let Err(e) = result {
            assert!(false, "清空历史记录失败 {}", e);
        }
    }

    #[test]
    fn test_remove_open_hisotry() {
        let data_path = PathBuf::from("/home/liuning/tmp");
        let result = remove_open_history(data_path, 1);
        if let Err(e) = result {
            assert!(false, "移除历史记录失败 {}", e);
        }
    }
}
