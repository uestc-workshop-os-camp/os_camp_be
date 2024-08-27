use std::collections::HashMap;

use crate::models::user_info::{self, insert};
use base64::decode;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const TOKEN: &str = "";
const ORGANIZER: &str = "uestc-workshop-os-camp";

#[derive(Debug, Serialize, Deserialize)]
struct Repo {
    id: u64,
    name: String,
    owner: Owner,
}

#[derive(Debug, Serialize, Deserialize)]
struct Owner {
    avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonFile {
    content: String,
}

/**
 * flush database data
 */
pub async fn get_score() {
    use tokio::time::{interval, Duration};

    let mut interval = interval(Duration::from_secs(3600)); // 1h

    // format url
    let url = format!("https://api.github.com/orgs/{}/repos", ORGANIZER);
    // set header
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(TOKEN).unwrap(),
    );
    // Create a new HTTP client with the headers
    let client = Client::builder().default_headers(headers).build().unwrap();

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
                                let latest_json_url = format!(
                                    "https://api.github.com/repos/{}/{}/contents/latest.json",
                                    username, repo.name
                                );

                                match client.get(&latest_json_url).send().await {
                                    Ok(response) => {
                                        if response.status().is_success() {
                                            let latest_json_file: JsonFile =
                                                response.json().await.unwrap();
                                            // 解码 Base64 字符串
                                            let decoded_bytes =
                                                decode(&latest_json_file.content).unwrap();

                                            // 将解码后的字节序列转换为字符串
                                            let decoded_str =
                                                String::from_utf8(decoded_bytes).unwrap();

                                            // 解析字符串为 JSON
                                            let json_data: Value =
                                                serde_json::from_str(&decoded_str).unwrap();

                                            if let Value::Object(map) = json_data {
                                                let data: HashMap<String, Value> =
                                                    map.into_iter().collect();

                                                let mut user_info = user_info::UserInfo::new();

                                                for (key, value) in &data {
                                                    let score_file_url = format!("https://api.github.com/repos/{}/{}/contents/{}",username,repo.name,value);

                                                    match client.get(&score_file_url).send().await {
                                                        Ok(res) => {
                                                            if res.status().is_success() {
                                                                let file: JsonFile =
                                                                    res.json().await.unwrap();
                                                                // 解码 Base64 字符串
                                                                // 预处理，去掉换行符
                                                                let file_content =
                                                                    file.content.replace("\n", "");
                                                                let decoded =
                                                                    match decode(&file_content) {
                                                                        Ok(decode) => decode,
                                                                        Err(err) => {
                                                                            eprintln!(
                                                                            "Failed to decode: {}",
                                                                            err
                                                                        );
                                                                            continue;
                                                                        }
                                                                    };
                                                                // 将解码后的字节序列转换为字符串
                                                                let decoded_string =
                                                                    String::from_utf8(decoded)
                                                                        .unwrap();
                                                                // 按空格分割字符串
                                                                let parts: Vec<&str> =
                                                                    decoded_string
                                                                        .split_whitespace()
                                                                        .collect();
                                                                let score_part = parts[1];
                                                                // 按"/"分割以获取分子和分母
                                                                let score_split: Vec<&str> =
                                                                    score_part.split('/').collect();
                                                                let numerator: f64 =
                                                                    score_split[0].parse().unwrap();
                                                                let denominator: f64 =
                                                                    score_split[1].parse().unwrap();
                                                                // 计算小数形式的分数
                                                                let score = (numerator
                                                                    / denominator)
                                                                    * 100.0;

                                                                match key.as_str() {
                                                                    "ch3" => {
                                                                        user_info.ch3 = score
                                                                    }
                                                                    "ch4" => {
                                                                        user_info.ch4 = score
                                                                    }
                                                                    "ch5" => {
                                                                        user_info.ch5 = score
                                                                    }
                                                                    "ch6" => {
                                                                        user_info.ch6 = score
                                                                    }
                                                                    "ch8" => {
                                                                        user_info.ch8 = score
                                                                    }
                                                                    _ => (),
                                                                }
                                                            }
                                                        }
                                                        Err(err) => {
                                                            eprintln!("Failed: {}", err);
                                                        }
                                                    }
                                                }
                                                user_info.username = username.to_string();
                                                // 通过 github rest api 由用户名称获取用户信息
                                                let github_user_info_url = format!(
                                                    "https://api.github.com/users/{}",
                                                    username
                                                );
                                                user_info.header_url = match client
                                                    .get(&github_user_info_url)
                                                    .send()
                                                    .await
                                                {
                                                    Ok(res) => {
                                                        if res.status().is_success() {
                                                            //解析json
                                                            let user_info: Owner =
                                                                res.json().await.unwrap();
                                                            // debug 打印 用户信息
                                                            println!("{:?}", user_info);
                                                            // 返回json中有avatar_url
                                                            user_info.avatar_url
                                                        } else {
                                                            "".to_string()
                                                        }
                                                    }
                                                    Err(err) => {
                                                        eprintln!(
                                                            "Failed to fetch user info: {}",
                                                            err
                                                        );
                                                        "".to_string()
                                                    }
                                                };
                                                println!("{:?}", user_info);
                                                // 插入数据库
                                                if let Err(e) =
                                                    insert(&user_info)
                                                {
                                                    eprintln!("Failed to insert data: {:?}", e);
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("Failed: {}", err);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    eprintln!("Failed to fetch repos: {}", response.status());
                }
            }
            Err(err) => {
                eprintln!("Failed to fetch repos: {}", err);
            }
        }
    }
}
