use crate::models::user_info::{self, phase1_insert, phase2_insert};
use crate::models::user_info::{Phase1UserInfo, Phase2UserInfo};
use base64::decode;
use chrono::{FixedOffset, NaiveDateTime};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const ORGANIZER: &str = "uestc-workshop-os-camp";
const TIME_FORMAT: &str = "%Y_%m_%d_%H_%M_%S";
const HOUR: i32 = 3600;
const ZONE: i32 = 8;
lazy_static::lazy_static!(
    pub static ref DEFAULT_TIME:i64 = std::env::var("DEFAULT_HAND_TIME").unwrap_or("1725174292".to_string()).parse::<i64>().unwrap();
);

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
    // 从环境变量中获取 delay时间, 默认为30s，将字符串转换为u64
    let delay = std::env::var("DELAY")
        .unwrap_or("30".to_string())
        .parse::<u64>()
        .unwrap();
    let mut interval = interval(Duration::from_secs(delay));
    let token: String = std::env::var("PERSONAL_GITHUB_TOKEN").expect("github TOKEN must be set");
    let token = "Bearer ".to_string() + &token;
    // set header
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&token).unwrap(),
    );
    // debug 打印 TOKEN
    println!("TOKEN: {}", token);
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
        // format url
        let mut page = 1;
        let mut url = format!(
            "https://api.github.com/orgs/{}/repos?per_page=100&page={}",
            ORGANIZER, page
        );
        // task
        // Send a GET request to the URL
        loop {
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        // 如果内容为空，则退出循环
                        if response.content_length().unwrap() == 2 {
                            // debug 打印
                            println!("No more repos");
                            break;
                        }
                        // 解析 JSON 响应
                        let repos: Vec<Repo> = response.json().await.unwrap();
                        for repo in repos {
                            #[cfg(any(
                                feature = "rcore-camp-score",
                                feature = "rcore-rustlings-score"
                            ))]
                            insert_score_info(repo, &client).await;
                        }
                    } else {
                        eprintln!(
                            "send success butFailed to fetch repos: {}",
                            response.status()
                        );
                    }
                }
                Err(err) => {
                    eprintln!("Failed to fetch repos: {}", err);
                }
            }
            page += 1;
            url = format!(
                "https://api.github.com/orgs/{}/repos?per_page=100&page={}",
                ORGANIZER, page
            );
        }
    }
}
#[cfg(any(feature = "rcore-camp-score", feature = "rcore-rustlings-score"))]
async fn insert_score_info(repo: Repo, client: &Client) {
    if repo.name.starts_with("rcore-camp-") || repo.name.starts_with("rcore-rustlings-") {
        // 格式化最新文件的 URL
        let latest_json_url = format!(
            "https://api.github.com/repos/{}/{}/contents/latest.json?ref=gh-pages",
            ORGANIZER, repo.name
        );
        // debug 打印请求的url
        // if repo.name.starts_with("rcore-camp-") {
        //     println!("rcore-camp-: {}", latest_json_url);
        // }

        match client.get(&latest_json_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let latest_json_file: JsonFile = response.json().await.unwrap();
                    // 解码 Base64 字符串
                    // 预处理，去掉换行符
                    let latest_json_file_content = latest_json_file.content.replace("\n", "");
                    let decoded_bytes = match decode(&latest_json_file_content) {
                        Ok(decode) => decode,
                        Err(err) => {
                            eprintln!("Failed to decode: {}", err);
                            return;
                        }
                    };

                    // 将解码后的字节序列转换为字符串
                    let decoded_str = String::from_utf8(decoded_bytes).unwrap();

                    // 解析字符串为 JSON
                    let json_data: Value = serde_json::from_str(&decoded_str).unwrap();

                    if let Value::Object(map) = json_data {
                        let data: HashMap<String, Value> = map.into_iter().collect();

                        #[cfg(feature = "rcore-camp-score")]
                        let mut phase2_user_info = user_info::Phase2UserInfo::new();
                        #[cfg(feature = "rcore-rustlings-score")]
                        let mut phase1_user_info = user_info::Phase1UserInfo::new();
                        let mut timestamp = 0;
                        let mut username = "".to_string();
                        let mut phase: u8 = 0;
                        for (key, value) in &data {
                            // 预处理，去掉key和value包裹的引号
                            let key = key.replace("\"", "");
                            let value = value.to_string().replace("\"", "");
                            // 将value的.txt去掉,得到文件名，格式为2024_08_20_09_11_05, 2024年8月20日9点11分05秒,从而转换为时间戳
                            let time = value.replace(".txt", "");
                            // 将时间戳转换为时间
                            timestamp = match NaiveDateTime::parse_from_str(&time, TIME_FORMAT) {
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
                                    eprintln!("Failed to parse time: {}", err);
                                    continue;
                                }
                            };

                            let score_file_url = format!(
                                "https://api.github.com/repos/{}/{}/contents/{}?ref=gh-pages",
                                ORGANIZER, repo.name, value
                            );

                            match client.get(&score_file_url).send().await {
                                Ok(res) => {
                                    if res.status().is_success() {
                                        let file: JsonFile = res.json().await.unwrap();
                                        // 预处理，去掉换行符
                                        let file_content = file.content.replace("\n", "");
                                        // 解码 Base64 字符串
                                        let decoded = match decode(file_content) {
                                            Ok(decode) => decode,
                                            Err(err) => {
                                                eprintln!("Failed to decode: {}", err);
                                                return;
                                            }
                                        };
                                        // 将解码后的字节序列转换为字符串
                                        let decoded_string = String::from_utf8(decoded).unwrap();
                                        #[cfg(feature = "rcore-camp-score")]
                                        if let Some(_username) =
                                            repo.name.strip_prefix("rcore-camp-")
                                        {
                                            phase2_rcore_camp_score(
                                                &mut phase2_user_info,
                                                &decoded_string,
                                                &key,
                                            );
                                            username = _username.to_string();
                                            phase = 2;
                                        }
                                        #[cfg(feature = "rcore-rustlings-score")]
                                        if let Some(_username) =
                                            repo.name.strip_prefix("rcore-rustlings-")
                                        {
                                            phase1_rcore_rustring_score(
                                                &mut phase1_user_info,
                                                &decoded_string,
                                            );
                                            username = _username.to_string();
                                            phase = 1;
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
                        let github_user_header_url = get_user_header_url(&username, client).await;
                        #[cfg(feature = "rcore-camp-score")]
                        if phase == 2 {
                            phase2_user_info.header_url = github_user_header_url.clone();
                            phase2_user_info.username = username.to_string();

                            phase2_user_info.pass_time = timestamp;
                            // 算总分
                            phase2_user_info.total = phase2_user_info.ch3
                                + phase2_user_info.ch4
                                + phase2_user_info.ch5
                                + phase2_user_info.ch6
                                + phase2_user_info.ch8;
                            // debug 打印用户信息
                            println!("{:?}", phase2_user_info);
                            // 插入数据库
                            if let Err(e) = phase2_insert(&phase2_user_info) {
                                eprintln!("Failed to insert data: {:?}", e);
                            }
                        }
                        #[cfg(feature = "rcore-rustlings-score")]
                        if phase == 1 {
                            phase1_user_info.header_url = github_user_header_url.clone();
                            phase1_user_info.username = username.to_string();
                            phase1_user_info.pass_time = timestamp;
                            // debug 打印用户信息
                            println!("{:?}", phase1_user_info);
                            // 插入数据库
                            if let Err(e) = phase1_insert(&phase1_user_info) {
                                eprintln!("Failed to insert data: {:?}", e);
                            }
                        }
                    }
                } else if response.status().as_u16() != 404 {
                    // 这种情况下，不可能是因为 gh-pages 分支没有 latest.json 文件，不可以接受
                    eprintln!(
                        "Failed to fetch latest json file: {}",
                        response.text().await.unwrap()
                    );
                } else {
                    // 404 Not Found的情况，插入全0分数
                    #[cfg(feature = "rcore-camp-score")]
                    if let Some(_username) = repo.name.strip_prefix("rcore-camp-") {
                        let mut phase2_user_info = user_info::Phase2UserInfo::new();
                        phase2_user_info.username = _username.to_string();
                        phase2_user_info.ch3 = 0.0;
                        phase2_user_info.ch4 = 0.0;
                        phase2_user_info.ch5 = 0.0;
                        phase2_user_info.ch6 = 0.0;
                        phase2_user_info.ch8 = 0.0;
                        phase2_user_info.total = 0.0;
                        phase2_user_info.pass_time = DEFAULT_TIME.clone();
                        phase2_user_info.header_url = get_user_header_url(_username, client).await;
                        // debug 打印用户信息
                        println!("{:?}", phase2_user_info);
                        if let Err(e) = phase2_insert(&phase2_user_info) {
                            eprintln!("Failed to insert data: {:?}", e);
                        }
                    }

                    #[cfg(feature = "rcore-rustlings-score")]
                    if let Some(_username) = repo.name.strip_prefix("rcore-rustlings-") {
                        let mut phase1_user_info = user_info::Phase1UserInfo::new();
                        phase1_user_info.username = _username.to_string();
                        phase1_user_info.points = 0.0;
                        phase1_user_info.total = 0.0;
                        phase1_user_info.pass_time = DEFAULT_TIME.clone();
                        phase1_user_info.header_url = get_user_header_url(_username, client).await;
                        // debug 打印用户信息
                        println!("{:?}", phase1_user_info);
                        if let Err(e) = phase1_insert(&phase1_user_info) {
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

/**
 * 获取 rcore-rustlings- 仓库的分数
 */
#[cfg(feature = "rcore-rustlings-score")]
fn phase1_rcore_rustring_score(phase1_user_info: &mut Phase1UserInfo, decoded_string: &String) {
    //获取以Points: 开头的行
    let points_line = decoded_string
        .lines()
        .find(|line| line.starts_with("Points:"))
        .unwrap();
    // 从这一行，比如 Points: 1/110 中提取分数
    let points: Vec<&str> = points_line.split_whitespace().collect();
    let points_split: Vec<&str> = points[1].split('/').collect();
    let numerator: f64 = points_split[0].parse().unwrap();
    let denominator: f64 = points_split[1].parse().unwrap();
    phase1_user_info.points = numerator;
    phase1_user_info.total = denominator;
}

/**
 * 获取 rcore-camp- 仓库的分数
 */
#[cfg(feature = "rcore-camp-score")]
fn phase2_rcore_camp_score(
    phase2_user_info: &mut Phase2UserInfo,
    decoded_string: &String,
    key: &str,
) {
    // 按空格分割字符串
    let parts: Vec<&str> = decoded_string.split_whitespace().collect();
    let score_part = parts[1];
    // 按"/"分割以获取分子和分母
    let score_split: Vec<&str> = score_part.split('/').collect();
    let numerator: f64 = score_split[0].parse().unwrap();
    let denominator: f64 = score_split[1].parse().unwrap();
    // 计算小数形式的分数
    let score = (numerator / denominator) * 100.0;

    match key {
        "ch3" => phase2_user_info.ch3 = score,
        "ch4" => phase2_user_info.ch4 = score,
        "ch5" => phase2_user_info.ch5 = score,
        "ch6" => phase2_user_info.ch6 = score,
        "ch8" => phase2_user_info.ch8 = score,
        _ => (),
    }
}

/**
 * 获取用户头像
 */

async fn get_user_header_url(username: &str, client: &Client) -> String {
    // 通过 github rest api 由用户名称获取用户信息
    let github_user_info_url = format!("https://api.github.com/users/{}", username);
    match client.get(&github_user_info_url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                //解析json
                let user_info: Owner = res.json().await.unwrap();
                // debug 打印 用户信息
                println!("{:?}", user_info);
                // 返回json中有avatar_url
                user_info.avatar_url
            } else {
                "".to_string()
            }
        }
        Err(err) => {
            eprintln!("Failed to fetch user info: {}", err);
            "".to_string()
        }
    }
}
