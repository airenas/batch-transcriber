use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::model::schema::work_data)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WorkData {
    pub id: String,
    pub external_id: String,
    pub file_name: String,
    pub base_dir: String,
    pub try_count: i32,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime
}
