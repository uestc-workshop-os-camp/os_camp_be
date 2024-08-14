use serde::{Deserialize, Serialize};
use diesel::{dsl::{insert_into, sum}, prelude::{Insertable, Queryable}, ExpressionMethods, QueryDsl, RunQueryDsl, Selectable};
use crate::config::database::establish_connection;
// use crate::schema::user_info::dsl::user_info;

#[derive(Debug,Deserialize,Serialize,Insertable,Selectable,Queryable)]
#[diesel(table_name = crate::schema::user_info)]
#[serde(crate = "rocket::serde")]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct UserInfo {
    pub id: Option<u32>,
    pub username: String,
    pub header_url: String,
    pub ch3: i32,
    pub ch4: i32,
    pub ch5: i32,
    pub ch6: i32,
    pub ch8: i32,
}

impl UserInfo {
    pub fn page(offset: i32,limit: i32) -> Result<Vec<UserInfo>, diesel::result::Error> {
        use crate::schema::user_info::dsl::*;

        let conn_poll =&mut establish_connection();
        let conn = &mut conn_poll.get().unwrap();
        let results = user_info
        .order_by(sum(ch3 + ch4 + ch5 + ch6 + ch8).desc()) // 按总成绩降序排列
        .limit(limit.into())
        .offset(offset.into())
        .load::<UserInfo>(conn)?;

        Ok(results)
    }

    pub fn new() -> UserInfo {
        UserInfo {
            id: None,// auto increase
            username: String::new(),
            header_url: String::new(),
            ch3: 0,
            ch4: 0,
            ch5: 0,
            ch6: 0,
            ch8: 0,
        }
    }

    pub fn insert(userInfo: &UserInfo) -> Result<(),diesel::result::Error> {
        use crate::schema::user_info::dsl::*;

        let conn_poll = &mut establish_connection();
        let conn = &mut conn_poll.get().unwrap();
        insert_into(user_info) 
            .values(userInfo)
            .execute(conn)?;
        Ok(())
    }
}