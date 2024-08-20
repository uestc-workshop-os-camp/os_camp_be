mod routes;
mod models;
mod config;
mod schema;
mod task;

use rocket::{catch, catchers, launch, serde::json::{json, Value},routes};
use rocket_cors::{Cors, CorsOptions,AllowedOrigins,AllowedHeaders};
use task::get_score_task::get_score;
use crate::routes::score::getScores;

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
		allowed_origins:AllowedOrigins::All,
		allowed_headers: AllowedHeaders::All,
		allow_credentials: true,
		..Default::default()
    }.to_cors()
    .expect("Cors fairing cannot be created")
}

// by noah: maybe non-async
#[launch]
pub async fn rocket() -> _ {
    // 创建 Rocket 实例
    let rocket = rocket::build()
    .mount("/api", routes![getScores])
    .attach(cors_fairing())
    .register("/", catchers![not_found]);

     // 启动定时任务
    rocket::tokio::spawn(async {
        get_score().await;
    });

    // 返回 Rocket 实例
    rocket
}