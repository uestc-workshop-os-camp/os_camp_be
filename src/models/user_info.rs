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
#[derive(Debug, Serialize)]
pub struct UserInfoPhase1 {
    pub id: u32,
    pub username: String,
    pub header_url: String,
    pub points: f64,
    pub total: f64,
    pub pass_time: i64,
    rank: i32,
}

#[derive(Debug, Serialize)]
pub struct UserInfoPhase2 {
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
    rank: i32,
}

pub fn phase1_page(offset: i32, limit: i32) -> Result<Vec<UserInfoPhase1>, diesel::result::Error> {
    use crate::schema::phase1_user_info::dsl::*;

    let conn = &mut conn_poll.get().unwrap();
    let results = phase1_user_info
        .order_by(points.desc()) // 按总成绩降序排列
        .then_order_by(pass_time.asc()) // 然后按通过时间升序排列
        .limit(limit.into())
        .offset(offset.into())
        .load::<Phase1UserInfo>(conn)?;
    let mut result_infos:Vec<UserInfoPhase1> = Vec::new();
    for (i, user) in results.into_iter().enumerate() {
        result_infos.push(UserInfoPhase1 {
            id: user.id,
            username: user.username,
            header_url: user.header_url,
            points: user.points,
            total: user.total,
            pass_time: user.pass_time,
            rank: (offset + i as i32 + 1),
        });
    }
    Ok(result_infos)
}

pub fn phase2_page(offset: i32, limit: i32) -> Result<Vec<UserInfoPhase2>, diesel::result::Error> {
    use crate::schema::phase2_user_info::dsl::*;

    let conn = &mut conn_poll.get().unwrap();
    let results = phase2_user_info
        .order_by(total.desc()) // 按总成绩降序排列
        .then_order_by(pass_time.asc()) // 然后按通过时间升序排列
        .limit(limit.into())
        .offset(offset.into())
        .load::<Phase2UserInfo>(conn)?;
    let mut result_infos:Vec<UserInfoPhase2> = Vec::new();
    for (i, user) in results.into_iter().enumerate() {
        result_infos.push(UserInfoPhase2 {
            id: user.id,
            username: user.username,
            header_url: user.header_url,
            ch3: user.ch3,
            ch4: user.ch4,
            ch5: user.ch5,
            ch6: user.ch6,
            ch8: user.ch8,
            total: user.total,
            pass_time: user.pass_time,
            rank: (offset + i as i32 + 1),
        });
    }
    Ok(result_infos)
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
