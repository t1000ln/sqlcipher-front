//! 请描述文件用途。
use api_resp::ApiResp;
use serde_json::json;

use support::history;
use support::history::get_open_history;

use crate::get_config_dir;

#[tauri::command]
pub async fn load_history() -> String {
    let data = get_open_history(get_config_dir());
    ApiResp::success(json!(data));
    // .to_json_str("请填写出错时的日志信息")
}
