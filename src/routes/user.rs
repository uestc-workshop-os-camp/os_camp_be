use std::collections::HashMap;

use reqwest::{header, Client};
use rocket::{
    post,
    serde::json::{json, Json, Value},
};
use serde::Deserialize;

#[derive(Deserialize)]
struct LoginVO {
    name: String,
    id: String,
    avatar: String,
}

const CLIENT_ID: &str = "Iv23li5eMNkhLTYR3wLU";
const CLIENT_SECRET: &str = "b5c5cac90a0ec07782f0bb0e4230340ead22f817";

#[post("/login/<code>", format = "json")]
pub async fn login(code: String) -> Json<Value> {
    // 拼接url
    let url = format!(
        "http://github.com/login/oauth/access_token?client_id={}&client_secret={}&code={}",
        CLIENT_ID, CLIENT_SECRET, code
    );
    // 发送post请求
    // 创建请求头
    let mut headers = header::HeaderMap::new();
    // 设置 Content-Type 请求头为 application/json
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    let client = Client::new();
    match client.post(&url).headers(headers).send().await {
        Ok(response) => {
            // 检查响应状态并解析响应体为 JSON
            if response.status().is_success() {
                match response.json::<HashMap<String, String>>().await {
                    Ok(json) => Json(serde_json::to_value(json).unwrap()), // 将响应转换为 JSON 格式返回
                    Err(_) => Json(json!({"error": "Failed to parse JSON response"})),
                }
            } else {
                Json(json!({"error": "Failed to authenticate"}))
            }
        }
        Err(_) => Json(json!({"error": "Request failed"})),
    }
}
