use crate::crypto::parse_uuid;
use crate::structs::general::DbPkLoadable;
use diesel::dsl::Find;
use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::query_dsl::filter_dsl::FindDsl;
use diesel::query_dsl::LoadQuery;
use diesel::r2d2::{ConnectionManager, Pool};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;

use r2d2::PooledConnection;

pub mod models;
pub mod schema;
pub mod views;

use bb8_redis::{bb8, RedisConnectionManager};

pub type Ppc = PooledConnection<ConnectionManager<PgConnection>>;
pub type RedisConnection = OnceCell<bb8::Pool<RedisConnectionManager>>;

lazy_static! {
    pub static ref PG_POOL: OnceCell<Pool<ConnectionManager<PgConnection>>> = OnceCell::new();
}

lazy_static! {
    pub static ref RS_POOL: OnceCell::<bb8::Pool<RedisConnectionManager>> = OnceCell::new();
}

pub fn get_pg_connection() -> PooledConnection<ConnectionManager<PgConnection>> {
    PG_POOL
        .get()
        .expect("Tried to access database before configured")
        .get()
        .expect("Error establishing database connection")
}

pub fn create_pg_connection_pool(db_url: &String) -> Pool<ConnectionManager<PgConnection>> {
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    Pool::builder()
        .max_size(20)
        .build(manager)
        .expect("Code: E100 - Error creating Database connection.")
}

pub async fn create_redis_connection_pool(url: &String) -> bb8::Pool<RedisConnectionManager> {
    info!("Checking Redis connectivity");
    let manager = RedisConnectionManager::new(url.clone()).unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();
    let pool_ = pool.clone();
    let mut connection = pool_.get().await.unwrap();
    bb8_redis::redis::cmd("PING")
        .query_async::<_, String>(&mut *connection)
        .await
        .expect("Redis PING failure");
    pool
}

pub fn get_db_object_by_id<'a, Model, Table, Key>(
    conn: &Ppc,
    table: Table,
    id: &'a Key,
) -> Result<Model, String>
where
    Table: FindDsl<&'a Key>,
    Find<Table, &'a Key>: LoadQuery<PgConnection, Model>,
    Key: Sized,
{
    let result = table.find(id).load::<Model>(conn);

    match result {
        Ok(mut data) => {
            if !data.is_empty() {
                Ok(data.pop().unwrap())
            } else {
                Err(format!("Row with provided key not found"))
            }
        }
        Err(e) => Err(format!("{}", e)),
    }
}

pub fn insert_db_object<Model, Table, Values>(
    conn: &Ppc,
    model_to_insert: Model,
    table: Table,
) -> Result<Model, String>
where
    Model: Insertable<Table, Values = Values>,
    InsertStatement<Table, Values>: LoadQuery<PgConnection, Model>,
{
    let result = model_to_insert.insert_into(table).get_result::<Model>(conn);

    match result {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("{}", e)),
    }
}

pub fn insert_db_object_dyn<ReturnModel, Model, Table, Values>(
    conn: &Ppc,
    model_to_insert: Model,
    table: Table,
) -> Result<ReturnModel, String>
where
    Model: Insertable<Table, Values = Values>,
    InsertStatement<Table, Values>: LoadQuery<PgConnection, ReturnModel>,
{
    let result = model_to_insert
        .insert_into(table)
        .get_result::<ReturnModel>(conn);

    match result {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("{}", e)),
    }
}

pub fn parse_and_load_uuid_pk<T>(uuid_str: &str) -> Result<T, ()>
where
    T: DbPkLoadable<PkType = uuid::Uuid, Output = std::result::Result<T, ()>>,
{
    if let Ok(uuid) = &parse_uuid(uuid_str) {
        T::load_pk(uuid)
    } else {
        Err(())
    }
}
