use super::super::schema::user_info;
use crate::config::database::conn_poll;
use diesel::{
    dsl::{insert_into, sum},
    prelude::{Insertable, Queryable},
    ExpressionMethods, QueryDsl, RunQueryDsl,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[diesel(table_name = user_info)]
#[serde(crate = "rocket::serde")]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct UserInfo {
    pub id: u32,
    pub username: String,
    pub header_url: String,
    pub ch3: f64,
    pub ch4: f64,
    pub ch5: f64,
    pub ch6: f64,
    pub ch8: f64,
}

impl UserInfo {
    pub fn new() -> UserInfo {
        UserInfo {
            id: 0, // auto increase
            username: String::new(),
            header_url: String::new(),
            ch3: 0.0,
            ch4: 0.0,
            ch5: 0.0,
            ch6: 0.0,
            ch8: 0.0,
        }
    }
}

pub fn page(offset: i32, limit: i32) -> Result<Vec<UserInfo>, diesel::result::Error> {
    use crate::schema::user_info::dsl::*;

    let conn = &mut conn_poll.get().unwrap();
    let results = user_info
        .order_by(sum(ch3 + ch4 + ch5 + ch6 + ch8).desc()) // 按总成绩降序排列
        .limit(limit.into())
        .offset(offset.into())
        .load::<UserInfo>(conn)?;

    Ok(results)
}

pub fn insert(user_info_: &UserInfo) -> Result<(), diesel::result::Error> {
    use crate::schema::user_info::dsl::*;

    let conn = &mut conn_poll.get().unwrap();
    // 如果主键（id、username）冲突则更新
    insert_into(user_info)
        .values(user_info_)
        .on_conflict(diesel::dsl::DuplicatedKeys)
        .do_update()
        .set((
            ch3.eq(user_info_.ch3),
            ch4.eq(user_info_.ch4),
            ch5.eq(user_info_.ch5),
            ch6.eq(user_info_.ch6),
            ch8.eq(user_info_.ch8),
        ))
        .execute(conn)?;
    Ok(())
}
