use super::super::schema::phase1_user_info;
use super::super::schema::phase2_user_info;
use crate::config::database::conn_poll;
use diesel::{
    dsl::insert_into,
    prelude::{Insertable, Queryable},
    ExpressionMethods, QueryDsl, RunQueryDsl,
};
use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[diesel(table_name = phase2_user_info)]
#[serde(crate = "rocket::serde")]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Phase2UserInfo {
    pub id: u32,
    pub username: String,
    pub header_url: String,
    pub ch3: f64,
    pub ch4: f64,
    pub ch5: f64,
    pub ch6: f64,
    pub ch8: f64,
    pub total: f64,
    pub pass_time: i64,
}

#[derive(Debug, Deserialize, Serialize, Queryable, Insertable)]
#[diesel(table_name = phase1_user_info)]
#[serde(crate = "rocket::serde")]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Phase1UserInfo {
    pub id: u32,
    pub username: String,
    pub header_url: String,
    pub points: f64,
    pub total: f64,
    pub pass_time: i64,
}

impl Phase2UserInfo {
    pub fn new() -> Phase2UserInfo {
        Phase2UserInfo {
            id: 2, // auto increase
            username: String::new(),
            header_url: String::new(),
            ch3: 0.0,
            ch4: 0.0,
            ch5: 0.0,
            ch6: 0.0,
            ch8: 0.0,
            total: 0.0,
            pass_time: i64::MAX,
        }
    }
}

impl Phase1UserInfo {
    pub fn new() -> Phase1UserInfo {
        Phase1UserInfo {
            id: 1, // auto increase
            username: String::new(),
            header_url: String::new(),
            points: 0.0,
            total: 0.0,
            pass_time: i64::MAX,
        }
    }
}

pub fn phase1_page(offset: i32, limit: i32) -> Result<Vec<Phase1UserInfo>, diesel::result::Error> {
    use crate::schema::phase1_user_info::dsl::*;

    let conn = &mut conn_poll.get().unwrap();
    let results = phase1_user_info
        .order_by(points.desc()) // 按总成绩降序排列
        .then_order_by(pass_time.asc()) // 然后按通过时间升序排列
        .limit(limit.into())
        .offset(offset.into())
        .load::<Phase1UserInfo>(conn)?;

    Ok(results)
}

pub fn phase2_page(offset: i32, limit: i32) -> Result<Vec<Phase2UserInfo>, diesel::result::Error> {
    use crate::schema::phase2_user_info::dsl::*;

    let conn = &mut conn_poll.get().unwrap();
    let results = phase2_user_info
        .order_by(total.desc()) // 按总成绩降序排列
        .then_order_by(pass_time.asc()) // 然后按通过时间升序排列
        .limit(limit.into())
        .offset(offset.into())
        .load::<Phase2UserInfo>(conn)?;

    Ok(results)
}

pub fn phase2_insert(user_info_: &Phase2UserInfo) -> Result<(), diesel::result::Error> {
    use crate::schema::phase2_user_info::dsl::*;

    let conn = &mut conn_poll.get().unwrap();
    // 如果主键（id、username）冲突则更新
    insert_into(phase2_user_info)
        .values(user_info_)
        .on_conflict(diesel::dsl::DuplicatedKeys)
        .do_update()
        .set((
            ch3.eq(user_info_.ch3),
            ch4.eq(user_info_.ch4),
            ch5.eq(user_info_.ch5),
            ch6.eq(user_info_.ch6),
            ch8.eq(user_info_.ch8),
            total.eq(user_info_.total),
            pass_time.eq(user_info_.pass_time),
        ))
        .execute(conn)?;
    Ok(())
}

pub fn phase1_insert(user_info_: &Phase1UserInfo) -> Result<(), diesel::result::Error> {
    use crate::schema::phase1_user_info::dsl::*;

    let conn = &mut conn_poll.get().unwrap();
    // 如果主键（id、username）冲突则更新
    insert_into(phase1_user_info)
        .values(user_info_)
        .on_conflict(diesel::dsl::DuplicatedKeys)
        .do_update()
        .set((
            points.eq(user_info_.points),
            total.eq(user_info_.total),
            pass_time.eq(user_info_.pass_time),
        ))
        .execute(conn)?;
    Ok(())
}
