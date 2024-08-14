use os_web;
use rocket;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    os_web::rocket().await.launch().await?;
    Ok(())
}