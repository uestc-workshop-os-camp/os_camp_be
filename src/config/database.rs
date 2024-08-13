use diesel::r2d2::{ConnectionManager, Pool};
use diesel::mysql::MysqlConnection;

/**
 * 数据库连接配置，到时候换成服务器上的用户名和密码
 */
pub fn establish_connection() -> Pool<ConnectionManager<MysqlConnection>> {
    Pool::builder()
        .build(ConnectionManager::<MysqlConnection>::new("mysql://root:YuanRui20050614@localhost/os_web"))
        .expect("Failed to create connection pool.")
}
