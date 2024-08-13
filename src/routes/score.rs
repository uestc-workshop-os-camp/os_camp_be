use rocket::serde::json::{json, Json, Value};
use diesel::mysql::MysqlConnection;

mod models;

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
pub async fn getScores(conn: &MysqlConnection,offset: i32,limit: i32,mode: i32) -> Value {
    
}