use rocket::serde::json::{json, Json, Value};

#[post("/login/<code>",format = "json")]
pub async fn login(code: String) -> Value {
    Json("login");
} 