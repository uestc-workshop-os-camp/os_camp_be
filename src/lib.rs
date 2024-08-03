#[macro_use]
extern crate rocket;

use rocket::{catch, catchers, launch, serde::json::{json, Value}};
use rocket_cors::{Cors, CorsOptions,AllowedOrigins,AllowedHeaders};

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

#[launch]
pub fn rocket() -> _{
    rocket::build()
    .mount("/api", routes![])
    .attach(cors_fairing())
    .register("/", catchers![not_found])
}