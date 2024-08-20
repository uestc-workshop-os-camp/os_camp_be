use std::collections::HashMap;

use reqwest::{Client,header};
use base64::decode;
use serde_json::Value;
use crate::models::user_info;
use serde::{Deserialize, Serialize};

const TOKEN: &str = "Bearer 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";
const ORGANIZER: &str = "uestc-workshop-os-camp";

#[derive(Debug, Serialize, Deserialize)]
struct Repo {
    id: u64,
    name: String,
    owner: Owner,
}

#[derive(Debug, Serialize, Deserialize)]
struct Owner {
    avatar_url: String
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonFile {
    content: String
}


/**
 * flush database data
 */
pub async fn get_score() {
    use tokio::time::{interval, Duration};

    let mut interval = interval(Duration::from_secs(3600));// 1h

    // format url
    let url = format!("https://api.github.com/orgs/{}/repos",ORGANIZER);
    // set header
    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION,header::HeaderValue::from_str(TOKEN).unwrap());
    // Create a new HTTP client with the headers
    let client = Client::builder()
    .default_headers(headers)
    .build().unwrap();

    loop {
        interval.tick().await;
        // task
        // Send a GET request to the URL
        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    // 解析 JSON 响应
                    let repos: Vec<Repo> = response.json().await.unwrap();

                    for repo in repos {
                        if repo.name.starts_with("rcore-camp-") {
                            // 去掉前缀 "rcore-camp-" 以获取用户名
                            if let Some(username) = repo.name.strip_prefix("rcore-camp-") {
                                // 格式化最新文件的 URL
                                let latest_json_url = format!("https://api.github.com/repos/{}/{}/contents/latest.json", username, repo.name);
                                
                                match client.get(&latest_json_url).send().await {
                                    Ok(response) => {
                                        if response.status().is_success() {
                                            let latest_json_file: JsonFile = response.json().await.unwrap();
                                            // 解码 Base64 字符串
                                            let decoded_bytes = decode(&latest_json_file.content).unwrap();

                                            // 将解码后的字节序列转换为字符串
                                            let decoded_str = String::from_utf8(decoded_bytes).unwrap();

                                            // 解析字符串为 JSON
                                            let json_data: Value = serde_json::from_str(&decoded_str).unwrap();

                                            if let Value::Object(map) = json_data {
                                                let data: HashMap<String, Value> = map.into_iter().collect();
                                                
                                                let mut user_info = user_info::UserInfo::new();

                                                for (key,value) in &data {
                                                    let score_file_url = format!("https://api.github.com/repos/{}/{}/contents/{}",username,repo.name,value);

                                                    match client.get(&score_file_url).send().await {
                                                        Ok(res) => {
                                                            if res.status().is_success() {
                                                                let file: JsonFile = res.json().await.unwrap();
                                                                // 解码 Base64 字符串
                                                                let decoded = decode(&file.content).unwrap();

                                                                // 将解码后的字节序列转换为字符串
                                                                let decoded_string = String::from_utf8(decoded).unwrap();
                                                                // 按空格分割字符串
                                                                let parts: Vec<&str> = decoded_string.split_whitespace().collect();
                                                                let score_part = parts[1];
                                                                // 按"/"分割以获取分子和分母
                                                                let score_split: Vec<&str> = score_part.split('/').collect();
                                                                let numerator: f64 = score_split[0].parse().unwrap();
                                                                let denominator: f64 = score_split[1].parse().unwrap();
                                                                // 计算小数形式的分数
                                                                let score = (numerator / denominator) * 100.0;

                                                                user_info.username = username.to_string();
                                                                user_info.header_url = repo.owner.avatar_url.clone();
                                                                match key.as_str() {
                                                                    "ch3" => user_info.ch3 = score as i32,
                                                                    "ch4" => user_info.ch4 = score as i32,
                                                                    "ch5" => user_info.ch5 = score as i32,
                                                                    "ch6" => user_info.ch6 = score as i32,
                                                                    "ch8" => user_info.ch8 = score as i32,
                                                                    _ => (),
                                                                }
                                                                
                                                                // 插入数据库
                                                                if let Err(e) = user_info::UserInfo::insert(&user_info) {
                                                                    eprintln!("Failed to insert data: {:?}", e);
                                                                }
                                                            }
                                                        }
                                                        Err(err)=>{
                                                            eprintln!("Failed: {}", err);
                                                        }
                                                    }
                                                }
                                            }
                                            // else {
                                            //     Err("Expected a JSON object".into())
                                            // }
                                        }
                                    }
                                    Err(err)=>{
                                        eprintln!("Failed: {}", err);
                                    }
                                }
                            }
                        }
                    }
                }else{
                    eprintln!("Failed to fetch repos: {}", response.status());
                }
            }
            Err(err) => {
                eprintln!("Failed to fetch repos: {}", err);
            }
        }
    }
}