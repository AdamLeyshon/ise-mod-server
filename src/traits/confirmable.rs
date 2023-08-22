use crate::db::models::bind::{AccountBind, ClientBind};

pub trait Confirmable {
    fn confirm(&mut self);
}

impl Confirmable for AccountBind {
    fn confirm(&mut self) {
        use crate::db::get_pg_connection;
        use crate::db::schema::account_binds as ab_schema;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        let conn = get_pg_connection();
        diesel::update(ab_schema::table.filter(ab_schema::bind_id.eq(&self.bind_id)))
            .set(ab_schema::confirmed.eq(true))
            .execute(&conn)
            .expect("Failed to mark account bind confirmed");
        self.confirmed = true;
    }
}

impl Confirmable for ClientBind {
    fn confirm(&mut self) {
        use crate::db::get_pg_connection;
        use crate::db::schema::client_binds as cb_schema;
        use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
        let conn = get_pg_connection();
        diesel::update(cb_schema::table.filter(cb_schema::client_bind_id.eq(&self.client_bind_id)))
            .set(cb_schema::confirmed.eq(true))
            .execute(&conn)
            .expect("Failed to mark account bind confirmed");
        self.confirmed = true;
    }
}
