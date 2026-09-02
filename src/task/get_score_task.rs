use crate::models::user_info::{phase1_insert, phase2_insert, Phase1UserInfo, Phase2UserInfo};
use base64::decode;
use chrono::{FixedOffset, NaiveDateTime, Utc};
use reqwest::{header, Client, StatusCode};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

const ORGANIZER: &str = "uestc-workshop-os-camp";
const TIME_FORMAT: &str = "%Y_%m_%d_%H_%M_%S";
const DEFAULT_REFRESH_INTERVAL_SECONDS: u64 = 15 * 60;
const DEFAULT_CONNECT_TIMEOUT_SECONDS: u64 = 5;
const DEFAULT_REQUEST_TIMEOUT_SECONDS: u64 = 20;
const CLIENT_RETRY_SECONDS: u64 = 60;
const HOUR: i32 = 3600;
const ZONE: i32 = 8;

static LAST_UPDATED_AT: AtomicI64 = AtomicI64::new(0);

lazy_static::lazy_static! {
    static ref DEFAULT_TIME: i64 = std::env::var("DEFAULT_HAND_TIME")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1_725_174_292);
}

type TaskResult<T> = Result<T, String>;

#[derive(Debug, Deserialize)]
struct Repo {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    avatar_url: String,
}

#[derive(Debug, Deserialize)]
struct JsonFile {
    content: String,
}

#[derive(Clone, Copy)]
enum ScorePhase {
    Rustlings,
    RCore,
}

pub fn last_updated_at() -> Option<i64> {
    match LAST_UPDATED_AT.load(Ordering::Relaxed) {
        0 => None,
        timestamp => Some(timestamp),
    }
}

pub fn refresh_interval_seconds() -> u64 {
    env_seconds("DELAY", DEFAULT_REFRESH_INTERVAL_SECONDS)
}

fn env_seconds(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn build_github_client() -> TaskResult<Client> {
    let token = std::env::var("PERSONAL_GITHUB_TOKEN")
        .map_err(|_| "PERSONAL_GITHUB_TOKEN is not configured".to_string())?;
    let authorization = header::HeaderValue::from_str(&format!("Bearer {token}"))
        .map_err(|error| format!("invalid GitHub authorization header: {error}"))?;

    let mut headers = header::HeaderMap::new();
    headers.insert(header::AUTHORIZATION, authorization);
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("spark-initiative-progress-service"),
    );
    headers.insert(
        header::HeaderName::from_static("x-github-api-version"),
        header::HeaderValue::from_static("2022-11-28"),
    );

    Client::builder()
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(env_seconds(
            "GITHUB_CONNECT_TIMEOUT_SECONDS",
            DEFAULT_CONNECT_TIMEOUT_SECONDS,
        )))
        .timeout(Duration::from_secs(env_seconds(
            "GITHUB_REQUEST_TIMEOUT_SECONDS",
            DEFAULT_REQUEST_TIMEOUT_SECONDS,
        )))
        .build()
        .map_err(|error| format!("failed to build GitHub client: {error}"))
}

/**
 * Periodically refresh leaderboard data without letting one bad repository
 * stop the whole refresh loop.
 */
pub async fn get_score() {
    let client = loop {
        match build_github_client() {
            Ok(client) => break client,
            Err(error) => {
                eprintln!("Unable to initialize GitHub client: {error}");
                tokio::time::sleep(Duration::from_secs(CLIENT_RETRY_SECONDS)).await;
            }
        }
    };

    let delay = refresh_interval_seconds();
    let mut interval = tokio::time::interval(Duration::from_secs(delay));
    let mut round_count: u64 = 0;

    loop {
        interval.tick().await;
        round_count += 1;

        match refresh_scores(&client).await {
            Ok(failed_repositories) => {
                let timestamp = Utc::now().timestamp();
                LAST_UPDATED_AT.store(timestamp, Ordering::Relaxed);
                if failed_repositories == 0 {
                    println!("Refresh round {round_count} completed");
                } else {
                    eprintln!(
                        "Refresh round {round_count} completed with {failed_repositories} repository error(s)"
                    );
                }
            }
            Err(error) => {
                eprintln!("Refresh round {round_count} failed: {error}");
            }
        }
    }
}

async fn refresh_scores(client: &Client) -> TaskResult<usize> {
    let mut page = 1;
    let mut failed_repositories = 0;

    loop {
        let repos = fetch_repositories(client, page).await?;
        if repos.is_empty() {
            break;
        }

        for repo in repos {
            if score_phase(&repo.name).is_none() {
                continue;
            }
            if let Err(error) = insert_score_info(repo, client).await {
                failed_repositories += 1;
                eprintln!("Failed to refresh repository: {error}");
            }
        }

        page += 1;
    }

    Ok(failed_repositories)
}

async fn fetch_repositories(client: &Client, page: u32) -> TaskResult<Vec<Repo>> {
    let url = format!(
        "https://api.github.com/orgs/{ORGANIZER}/repos?per_page=100&page={page}"
    );
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| format!("failed to fetch repository page {page}: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub returned {} for repository page {page}",
            response.status()
        ));
    }

    response
        .json::<Vec<Repo>>()
        .await
        .map_err(|error| format!("invalid repository page {page}: {error}"))
}

fn score_phase(repo_name: &str) -> Option<(ScorePhase, &str)> {
    if cfg!(feature = "rcore-camp-score") {
        if let Some(username) = repo_name.strip_prefix("rcore-camp-2026-") {
            return Some((ScorePhase::RCore, username));
        }
    }
    if cfg!(feature = "rcore-rustlings-score") {
        if let Some(username) = repo_name.strip_prefix("rcore-rustlings-2026-") {
            return Some((ScorePhase::Rustlings, username));
        }
    }
    None
}

async fn insert_score_info(repo: Repo, client: &Client) -> TaskResult<()> {
    let (phase, username) = match score_phase(&repo.name) {
        Some(value) => value,
        None => return Ok(()),
    };
    let username = username.to_string();
    let latest_json_url = format!(
        "https://api.github.com/repos/{ORGANIZER}/{}/contents/latest.json?ref=gh-pages",
        repo.name
    );
    let response = client
        .get(&latest_json_url)
        .send()
        .await
        .map_err(|error| format!("{}: failed to fetch latest.json: {error}", repo.name))?;

    if response.status() == StatusCode::NOT_FOUND {
        return insert_empty_score(phase, &username, client).await;
    }
    if !response.status().is_success() {
        return Err(format!(
            "{}: GitHub returned {} for latest.json",
            repo.name,
            response.status()
        ));
    }

    let latest_json_file = response
        .json::<JsonFile>()
        .await
        .map_err(|error| format!("{}: invalid latest.json response: {error}", repo.name))?;
    let latest_json = decode_json_file(latest_json_file, &repo.name)?;
    let data = serde_json::from_str::<HashMap<String, Value>>(&latest_json)
        .map_err(|error| format!("{}: invalid latest.json content: {error}", repo.name))?;
    if data.is_empty() {
        return Err(format!("{}: latest.json is empty", repo.name));
    }

    let mut phase1_user_info = Phase1UserInfo::new();
    let mut phase2_user_info = Phase2UserInfo::new();
    let mut timestamp = 0;
    let mut file_error = None;

    for (key, value) in data {
        let file_name = match value.as_str() {
            Some(value) => value,
            None => {
                file_error = Some(format!("{}: {key} does not name a score file", repo.name));
                continue;
            }
        };

        match parse_submission_time(file_name) {
            Ok(value) => timestamp = timestamp.max(value),
            Err(error) => {
                file_error = Some(format!("{}: {error}", repo.name));
                continue;
            }
        }

        let score_file_url = format!(
            "https://api.github.com/repos/{ORGANIZER}/{}/contents/{file_name}?ref=gh-pages",
            repo.name
        );
        let decoded_score = match fetch_json_content(client, &score_file_url, &repo.name).await {
            Ok(value) => value,
            Err(error) => {
                file_error = Some(error);
                continue;
            }
        };

        let parsed = match phase {
            ScorePhase::Rustlings => {
                phase1_rustlings_score(&mut phase1_user_info, &decoded_score)
            }
            ScorePhase::RCore => {
                phase2_rcore_camp_score(&mut phase2_user_info, &decoded_score, &key)
            }
        };
        if let Err(error) = parsed {
            file_error = Some(format!("{}: {error}", repo.name));
        }
    }

    if let Some(error) = file_error {
        return Err(error);
    }
    if timestamp == 0 {
        return Err(format!("{}: no valid submission time", repo.name));
    }

    let header_url = get_user_header_url(&username, client).await;
    match phase {
        ScorePhase::Rustlings => {
            phase1_user_info.header_url = header_url;
            phase1_user_info.username = username;
            phase1_user_info.pass_time = timestamp;
            phase1_insert(&phase1_user_info)
                .map_err(|error| format!("{}: database update failed: {error}", repo.name))
        }
        ScorePhase::RCore => {
            phase2_user_info.header_url = header_url;
            phase2_user_info.username = username;
            phase2_user_info.pass_time = timestamp;
            phase2_user_info.total = phase2_user_info.ch3
                + phase2_user_info.ch4
                + phase2_user_info.ch5
                + phase2_user_info.ch6
                + phase2_user_info.ch8;
            phase2_insert(&phase2_user_info)
                .map_err(|error| format!("{}: database update failed: {error}", repo.name))
        }
    }
}

async fn insert_empty_score(
    phase: ScorePhase,
    username: &str,
    client: &Client,
) -> TaskResult<()> {
    let header_url = get_user_header_url(username, client).await;
    match phase {
        ScorePhase::Rustlings => {
            let mut user_info = Phase1UserInfo::new();
            user_info.username = username.to_string();
            user_info.header_url = header_url;
            user_info.points = 0.0;
            user_info.total = 0.0;
            user_info.pass_time = *DEFAULT_TIME;
            phase1_insert(&user_info)
                .map_err(|error| format!("{username}: database update failed: {error}"))
        }
        ScorePhase::RCore => {
            let mut user_info = Phase2UserInfo::new();
            user_info.username = username.to_string();
            user_info.header_url = header_url;
            user_info.ch3 = 0.0;
            user_info.ch4 = 0.0;
            user_info.ch5 = 0.0;
            user_info.ch6 = 0.0;
            user_info.ch8 = 0.0;
            user_info.total = 0.0;
            user_info.pass_time = *DEFAULT_TIME;
            phase2_insert(&user_info)
                .map_err(|error| format!("{username}: database update failed: {error}"))
        }
    }
}

async fn fetch_json_content(client: &Client, url: &str, context: &str) -> TaskResult<String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("{context}: failed to fetch score file: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "{context}: GitHub returned {} for score file",
            response.status()
        ));
    }
    let file = response
        .json::<JsonFile>()
        .await
        .map_err(|error| format!("{context}: invalid score file response: {error}"))?;
    decode_json_file(file, context)
}

fn decode_json_file(file: JsonFile, context: &str) -> TaskResult<String> {
    let encoded = file.content.replace('\n', "");
    let bytes = decode(&encoded)
        .map_err(|error| format!("{context}: invalid base64 content: {error}"))?;
    String::from_utf8(bytes).map_err(|error| format!("{context}: content is not UTF-8: {error}"))
}

fn parse_submission_time(file_name: &str) -> TaskResult<i64> {
    let value = file_name.strip_suffix(".txt").unwrap_or(file_name);
    let time = NaiveDateTime::parse_from_str(value, TIME_FORMAT)
        .map_err(|error| format!("invalid submission time {file_name}: {error}"))?;
    let timezone = FixedOffset::east_opt(ZONE * HOUR)
        .ok_or_else(|| "invalid configured timezone".to_string())?;
    time.and_local_timezone(timezone)
        .single()
        .map(|value| value.timestamp())
        .ok_or_else(|| format!("ambiguous submission time {file_name}"))
}

fn phase1_rustlings_score(user_info: &mut Phase1UserInfo, content: &str) -> TaskResult<()> {
    let points_line = content
        .lines()
        .find(|line| line.starts_with("Points:"))
        .ok_or_else(|| "score file has no Points line".to_string())?;
    let score = points_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| "Points line has no score".to_string())?;
    let (numerator, denominator) = score
        .split_once('/')
        .ok_or_else(|| "Points score has no denominator".to_string())?;
    let numerator = numerator
        .parse::<f64>()
        .map_err(|error| format!("invalid Points numerator: {error}"))?;
    let denominator = denominator
        .parse::<f64>()
        .map_err(|error| format!("invalid Points denominator: {error}"))?;
    if denominator <= 0.0 {
        return Err("Points denominator must be positive".to_string());
    }
    user_info.points = numerator;
    user_info.total = denominator;
    Ok(())
}

fn phase2_rcore_camp_score(
    user_info: &mut Phase2UserInfo,
    content: &str,
    key: &str,
) -> TaskResult<()> {
    let score = content
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| format!("{key} score is missing"))?;
    let (numerator, denominator) = score
        .split_once('/')
        .ok_or_else(|| format!("{key} score has no denominator"))?;
    let numerator = numerator
        .parse::<f64>()
        .map_err(|error| format!("invalid {key} numerator: {error}"))?;
    let denominator = denominator
        .parse::<f64>()
        .map_err(|error| format!("invalid {key} denominator: {error}"))?;
    if denominator <= 0.0 {
        return Err(format!("{key} denominator must be positive"));
    }
    let score = (numerator / denominator) * 100.0;

    match key {
        "ch3" => user_info.ch3 = score,
        "ch4" => user_info.ch4 = score,
        "ch5" => user_info.ch5 = score,
        "ch6" => user_info.ch6 = score,
        "ch8" => user_info.ch8 = score,
        _ => return Err(format!("unknown chapter {key}")),
    }
    Ok(())
}

async fn get_user_header_url(username: &str, client: &Client) -> String {
    let url = format!("https://api.github.com/users/{username}");
    let response = match client.get(&url).send().await {
        Ok(response) => response,
        Err(error) => {
            eprintln!("Failed to fetch GitHub user {username}: {error}");
            return String::new();
        }
    };
    if !response.status().is_success() {
        eprintln!(
            "GitHub returned {} for user {username}",
            response.status()
        );
        return String::new();
    }
    match response.json::<GitHubUser>().await {
        Ok(user) => user.avatar_url,
        Err(error) => {
            eprintln!("Invalid GitHub user response for {username}: {error}");
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rustlings_score() {
        let mut user = Phase1UserInfo::new();
        assert!(phase1_rustlings_score(&mut user, "Progress\nPoints: 42/110\n").is_ok());
        assert_eq!(user.points, 42.0);
        assert_eq!(user.total, 110.0);
    }

    #[test]
    fn rejects_malformed_rustlings_score() {
        let mut user = Phase1UserInfo::new();
        assert!(phase1_rustlings_score(&mut user, "Progress only").is_err());
    }

    #[test]
    fn parses_rcore_score() {
        let mut user = Phase2UserInfo::new();
        assert!(phase2_rcore_camp_score(&mut user, "Score: 3/4", "ch3").is_ok());
        assert_eq!(user.ch3, 75.0);
    }

    #[test]
    fn parses_submission_timestamp() {
        assert_eq!(
            parse_submission_time("2026_09_02_12_34_56.txt"),
            Ok(1_788_323_696)
        );
    }
}
