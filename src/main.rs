use os_web;
use dotenv::dotenv;
use rocket;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().ok(); // 加载 .env 文件
    os_web::rocket().await.launch().await?;
    Ok(())
}
