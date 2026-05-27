use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "presets")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    pub class_id: i64,
    pub title: Option<String>,
    pub user_nickname: Option<String>,
    pub character_name: Option<String>,
    pub downloads: i64,
    pub views: i64,
    pub likes: i64,
    pub image_1: Option<String>,
    pub image_2: Option<String>,
    pub created_at: Option<i64>,
    pub customizing_id: Option<i64>,
    pub region: Option<String>,
    pub score: Option<i64>,
    pub pab_file: Option<String>,
    pub is_ok: i32,
    pub is_popular: i32,
    pub is_discarded: i32,
    pub is_wanted: i32,
    pub updated_at: Option<i64>,
    pub raw_json: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
