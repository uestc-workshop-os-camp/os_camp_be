use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use lazy_static::lazy_static;
lazy_static! {
    pub static ref conn_poll: Pool<ConnectionManager<MysqlConnection>> = establish_connection();
}

/**
 * 数据库连接配置，到时候换成服务器上的用户名和密码
 */
fn establish_connection() -> Pool<ConnectionManager<MysqlConnection>> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Pool::builder()
        .build(ConnectionManager::<MysqlConnection>::new(
            database_url,
        ))
        .expect("Failed to create connection pool.")
}
