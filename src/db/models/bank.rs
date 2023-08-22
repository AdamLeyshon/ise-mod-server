use crate::db::schema::bank_balances;
use uuid::Uuid;

#[derive(Queryable, Identifiable, Insertable, Debug, AsChangeset)]
#[primary_key(colony_id, currency)]
pub struct BankBalance {
    pub colony_id: Uuid,
    pub currency: i32,
    pub balance: i32,
}
