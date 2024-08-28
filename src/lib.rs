#![feature(trivial_bounds)]

mod config;
mod models;
mod routes;
mod schema;
mod task;

use crate::routes::score::get_scores;
use rocket::{
    catch, catchers, launch, routes,
    serde::json::{json, Value},
};
use rocket_cors::{AllowedHeaders, AllowedOrigins, Cors, CorsOptions};
use task::get_score_task::get_score;

// 处理404
#[catch(404)]
fn not_found() -> Value {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

// 配置跨域
fn cors_fairing() -> Cors {
    CorsOptions {
        allowed_origins: AllowedOrigins::All,
        allowed_headers: AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Cors fairing cannot be created")
}

// by noah: maybe non-async
#[launch]
pub async fn rocket() -> _ {
    // 创建 Rocket 实例
    let rocket = rocket::build()
        .mount("/api", routes![get_scores])
        .attach(cors_fairing())
        .register("/", catchers![not_found])
        .configure(rocket::Config {
            port: 8080,
            ..rocket::Config::default()
        });

    // 启动定时任务
    rocket::tokio::spawn(async {
        get_score().await;
    });

    // 返回 Rocket 实例
    rocket
}
