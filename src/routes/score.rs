use rocket::serde::json::{Json, Value};
use diesel::mysql::MysqlConnection;
use crate::models::user_info;

/**
 * 返回结果的结构体
 */
struct Result<T> {
    code: i32,// 0表示失败 1表示成功
    message: String, //错误信息，可以为空
    data: Vec<T>, // 排行榜信息
}

/**
 * 获取排行榜信息
 * offset: 起始页数
 * limit: 每页显示数量
 * mode: 哪一个排行榜 1表示ch3...ch8那个 2表示只有一个分数那个
 */
#[get("/scores/<offset>/<limit>/<mode>")]
pub async fn getScores(offset: i32,limit: i32,mode: i32) -> Value {
    let result = match mode {
        1 => {
            match user_info::UserInfo::page(offset, limit) {
                Ok(users) => {
                    let response = Result {
                        code: 1,
                        message: String::new(),
                        data: users,
                    };
                    Json(serde_json::json!(response))
                }
                Err(_) => {
                    let response = Result {
                        code: 0,
                        message: format!("数据库错误: {}", e),
                        data: Vec::new(),
                    };
                    Json(serde_json::json!(response))
                }
            }
        }
        2 => {
            // wait to impl
            let response = Result {
                code: 1,
                message: "模式 2 尚未实现".to_string(),
                data: Vec::new(),
            };
            Json(serde_json::json!(response))
        }
        _ => {
            let response = Result {
                code: 0,
                message: "无效的模式".to_string(),
                data: Vec::new(),
            };
            Json(serde_json::json!(response))
        }
    };

    result
}

pub mod score;