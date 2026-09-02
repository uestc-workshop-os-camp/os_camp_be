use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use lazy_static::lazy_static;
use std::error::Error;
use std::io;

type MysqlManager = ConnectionManager<MysqlConnection>;
type MysqlPool = Pool<MysqlManager>;

lazy_static! {
    static ref CONNECTION_POOL: Result<MysqlPool, String> = establish_connection();
}

/**
 * 数据库连接配置，到时候换成服务器上的用户名和密码
 */
fn establish_connection() -> Result<MysqlPool, String> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is not configured".to_string())?;
    Ok(Pool::builder().build_unchecked(MysqlManager::new(database_url)))
}

pub fn get_connection() -> Result<PooledConnection<MysqlManager>, Box<dyn Error + Send + Sync>> {
    let pool = CONNECTION_POOL
        .as_ref()
        .map_err(|error| io::Error::new(io::ErrorKind::NotConnected, error.clone()))?;
    Ok(pool.get()?)
}
