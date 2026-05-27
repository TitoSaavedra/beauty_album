use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "classes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id_garmoth: i64,
    pub id_pa: i64,
    pub display: String,
    pub icon_svg: Option<String>,
    pub is_favorite: i32,
    pub updated_at: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
