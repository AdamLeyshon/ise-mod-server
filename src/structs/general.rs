use semver_parser::version::Version;

pub const SERVER_VERSION: Version = Version {
    major: 1,
    minor: 3,
    patch: 0,
    pre: vec![],
    build: vec![],
};
pub const CLIENT_MIN_VERSION: Version = Version {
    major: 1,
    minor: 3,
    patch: 0,
    pre: vec![],
    build: vec![],
};
pub const CLIENT_MAX_VERSION: Version = Version {
    major: 1,
    minor: 9,
    patch: 9,
    pre: vec![],
    build: vec![],
};

pub const ONE_DAY_TICKS: i32 = 60_000;
pub const ONE_HOUR_TICKS: i32 = 2_500;

pub trait DbPkLoadable {
    type Output;
    type PkType;
    fn load_pk(pk: &Self::PkType) -> Self::Output;
}

#[macro_export]
macro_rules! make_pk_loadable {
    ($return_type:ty, $pk: ty, $schema:path) => {
        impl DbPkLoadable for $return_type {
            type Output = Result<Self, ()>;
            type PkType = $pk;
            fn load_pk(pk: &Self::PkType) -> Self::Output {
                use crate::db::{get_db_object_by_id, get_pg_connection};
                use $schema as schema;
                let conn = get_pg_connection();
                let result: Result<Self, String> = get_db_object_by_id(&conn, schema::table, pk);
                if let Ok(row) = result {
                    return Ok(row);
                } else {
                    Err(())
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_to_sql {
    (for $($t:ty),+) => {
        $(impl ToSql<Jsonb, Pg> for $t {
            fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> diesel::serialize::Result {
                let value = serde_json::to_value(self)?;
                <serde_json::Value as ToSql<Jsonb, Pg>>::to_sql(&value, out)
            }
        })*
    }
}

#[macro_export]
macro_rules! impl_from_sql {
    (for $($t:ty),+) => {
        $(impl FromSql<Jsonb, Pg> for $t {
            fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
                let value = <serde_json::Value as FromSql<Jsonb, Pg>>::from_sql(bytes)?;
                Ok(serde_json::from_value(value)?)
            }
        })*
    }
}
