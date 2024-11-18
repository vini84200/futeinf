use chrono::prelude::*;
use chrono::Duration;

pub const WEEK_IN_SECONDS: i64 = 604800;

pub fn get_first_ref_point() -> DateTime<Utc> {
    Local
        .with_ymd_and_hms(2023, 6, 4, 19, 0, 0)
        .unwrap()
        .with_timezone(&Utc)
}

pub fn get_ref_point_of(date: DateTime<Utc>) -> DateTime<Utc> {
    let past_reset = get_first_ref_point();

    //get Duration since reference reset
    let d = date - past_reset;

    // divide by a week, and get the remainder
    let rem = d.num_seconds() % WEEK_IN_SECONDS;

    //subtract the remainder from current time

    //last reset / datetime
    date - Duration::seconds(rem)
}

pub fn get_last_ref_point() -> DateTime<Utc> {
    let now = Utc::now();
    get_ref_point_of(now)
}

pub fn get_start_elegible_check(date: DateTime<Utc>) -> DateTime<Utc> {
    let last_reset = get_ref_point_of(date);
    last_reset - Duration::weeks(4)
}

pub fn get_end_elegible_check(date: DateTime<Utc>) -> DateTime<Utc> {
    get_ref_point_of(date)
}

pub fn get_start_voting(date: DateTime<Utc>) -> DateTime<Utc> {
    get_ref_point_of(date)
}

pub fn get_end_create_ballot(date: DateTime<Utc>) -> DateTime<Utc> {
    let last_reset = get_ref_point_of(date);
    last_reset + Duration::weeks(1) - Duration::minutes(30)
}

pub fn get_end_voting(date: DateTime<Utc>) -> DateTime<Utc> {
    let last_reset = get_ref_point_of(date);
    last_reset + Duration::weeks(1)
}

pub fn can_create_ballot(date: DateTime<Utc>) -> bool {
    let end = get_end_create_ballot(date);
    let start = get_start_voting(date);
    date >= start && date < end
}

pub fn can_cast_vote(rf: DateTime<Utc>) -> bool {
    let now = Utc::now();
    let start = get_start_voting(rf);
    let end = get_end_voting(rf);
    now >= start && now < end
}

pub fn publish_time(date: DateTime<Utc>) -> DateTime<Utc> {
    let end = get_end_voting(date);
    end + Duration::minutes(90)
}

pub fn publish_results(date: DateTime<Utc>) -> bool {
    let start_publish = publish_time(date);
    let now = Utc::now();
    now >= start_publish
}

pub fn ref_point_id(date: DateTime<Utc>) -> i32 {
    let last_reset = get_ref_point_of(date);
    let first_reset = get_first_ref_point();
    let weeks = (last_reset - first_reset).num_weeks();
    weeks as i32
}

pub fn ref_point_from_id(id: i32) -> DateTime<Utc> {
    let first_reset = get_first_ref_point();
    first_reset + Duration::weeks(id as i64)
}
