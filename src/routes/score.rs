use crate::models::user_info::{self, UserInfo};
#[allow(unused_imports)]
use diesel::mysql::MysqlConnection;
use rocket::{
    get,
    serde::json::{Json, Value},
};
use serde::Serialize;

/**
 * 返回结果的结构体
 */
#[derive(Debug, Serialize)]
struct Result {
    code: i32,                   // 0表示失败 1表示成功
    message: String,             //错误信息，可以为空
    data: Option<Vec<UserInfo>>, // 排行榜信息
}

/**
 * 获取排行榜信息
 * offset: 起始页数
 * limit: 每页显示数量
 * mode: 哪一个排行榜 1表示ch3...ch8那个 2表示只有一个分数那个
 */
#[get("/scores/<offset>/<limit>/<mode>")]
pub async fn get_scores(offset: i32, limit: i32, mode: i32) -> Json<Value> {
    let result: Json<Value> = match mode {
        1 => match user_info::UserInfo::page(offset, limit) {
            Ok(users) => {
                let response = Result {
                    code: 1,
                    message: String::new(),
                    data: Some(users),
                };
                Json(serde_json::json!(response))
            }
            Err(err) => {
                let response = Result {
                    code: 0,
                    message: format!("数据库错误: {}", err),
                    data: None,
                };
                Json(serde_json::json!(response))
            }
        },
        2 => {
            // wait to impl
            let response = Result {
                code: 1,
                message: "模式 2 尚未实现".to_string(),
                data: None,
            };
            Json(serde_json::json!(response))
        }
        _ => {
            let response = Result {
                code: 0,
                message: "无效的模式".to_string(),
                data: None,
            };
            Json(serde_json::json!(response))
        }
    };
    result
}
