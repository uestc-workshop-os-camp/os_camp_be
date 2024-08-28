use crate::models::user_info::{self, insert};
use base64::decode;
use chrono::{FixedOffset, NaiveDateTime};
use lazy_static::lazy_static;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// 使用 lazy_static 来延迟初始化 TOKEN
lazy_static! {
    static ref TOKEN: String = {
        "Bearer ".to_string() + &std::env::var("GITHUB_TOKEN").expect("github TOKEN must be set")
    };
}
const ORGANIZER: &str = "uestc-workshop-os-camp";
const TIME_FORMAT: &str = "%Y_%m_%d_%H_%M_%S";
const HOUR: i32 = 3600;
const ZONE: i32 = 0;

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

    let mut interval = interval(Duration::from_secs(30)); // 30s

    // format url
    let url = format!("https://api.github.com/orgs/{}/repos", ORGANIZER);
    // set header
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&TOKEN).unwrap(),
    );
    // debug 打印 TOKEN
    println!("TOKEN: {}", *TOKEN);
    // 设置 user-agent 请求头
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("reqwest"),
    );
    // Create a new HTTP client with the headers
    let client = Client::builder().default_headers(headers).build().unwrap();
    // debug: 查看轮数
    let mut round_count = 0;

    loop {
        interval.tick().await;
        round_count += 1;
        println!("Round: {}", round_count);
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
                                    "https://api.github.com/repos/{}/{}/contents/latest.json?ref=gh-pages",
                                    ORGANIZER, repo.name
                                );

                                match client.get(&latest_json_url).send().await {
                                    Ok(response) => {
                                        if response.status().is_success() {
                                            let latest_json_file: JsonFile =
                                                response.json().await.unwrap();
                                            // 解码 Base64 字符串
                                            // 预处理，去掉换行符
                                            let latest_json_file_content =
                                                latest_json_file.content.replace("\n", "");
                                            let decoded_bytes =
                                                match decode(&latest_json_file_content) {
                                                    Ok(decode) => decode,
                                                    Err(err) => {
                                                        eprintln!("Failed to decode: {}", err);
                                                        continue;
                                                    }
                                                };

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
                                                let mut timestamp = 0;

                                                for (key, value) in &data {
                                                    // 预处理，去掉key和value包裹的引号
                                                    let key = key.replace("\"", "");
                                                    let value = value.to_string().replace("\"", "");
                                                    // 将value的.txt去掉,得到文件名，格式为2024_08_20_09_11_05, 2024年8月20日9点11分05秒,从而转换为时间戳
                                                    let time = value.replace(".txt", "");
                                                    // 将时间戳转换为时间
                                                    timestamp = match NaiveDateTime::parse_from_str(
                                                        &time,
                                                        TIME_FORMAT,
                                                    ) {
                                                        Ok(time) => {

                                                            let tz = FixedOffset::east_opt(ZONE * HOUR).unwrap();
                                                            // 返回最老的时间戳
                                                            let tmp = time.and_local_timezone(tz).unwrap().timestamp();
                                                            if tmp < timestamp {
                                                                timestamp
                                                            } else {
                                                                tmp
                                                            }
                                                        }
                                                        Err(err) => {
                                                            eprintln!(
                                                                "Failed to parse time: {}",
                                                                err
                                                            );
                                                            continue;
                                                        }
                                                    };

                                                    let score_file_url = format!("https://api.github.com/repos/{}/{}/contents/{}?ref=gh-pages",ORGANIZER,repo.name,value);

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
                                                                    "ch3" => user_info.ch3 = score,
                                                                    "ch4" => user_info.ch4 = score,
                                                                    "ch5" => user_info.ch5 = score,
                                                                    "ch6" => user_info.ch6 = score,
                                                                    "ch8" => user_info.ch8 = score,
                                                                    _ => (),
                                                                }
                                                            } else {
                                                                eprintln!(
                                                                    "Failed to fetch json file: {}",
                                                                    res.text().await.unwrap()
                                                                );
                                                            }
                                                        }
                                                        Err(err) => {
                                                            eprintln!("Failed: {}", err);
                                                        }
                                                    }
                                                }
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
                                                user_info.username = username.to_string();
                                                // 因为是阶段2，所以id为2
                                                user_info.id = 2;
                                                user_info.pass_time = timestamp;
                                                // 算总分
                                                user_info.total = user_info.ch3
                                                    + user_info.ch4
                                                    + user_info.ch5
                                                    + user_info.ch6
                                                    + user_info.ch8;
                                                // debug 打印用户信息
                                                println!("{:?}", user_info);
                                                // 插入数据库
                                                if let Err(e) = insert(&user_info) {
                                                    eprintln!("Failed to insert data: {:?}", e);
                                                }
                                            }
                                        } else if response.status().as_u16() != 404 {
                                            // 这种情况下，不可能是因为 gh-pages 分支没有 latest.json 文件，不可以接受
                                            eprintln!(
                                                "Failed to fetch latest json file: {}",
                                                response.text().await.unwrap()
                                            );
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
                    eprintln!("send success butFailed to fetch repos: {}", response.status());
                }
            }
            Err(err) => {
                eprintln!("Failed to fetch repos: {}", err);
            }
        }
    }
}
