use crate::db::get_pg_connection;
use crate::db::models::order::Order;
use crate::db::models::trade_stats::TradeStatistic;
use crate::db::schema::trade_statistics as ts;
use crate::structs::general::DbPkLoadable;

use chrono::Utc;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use diesel::result::Error::RollbackTransaction;

use uuid::Uuid;

pub fn update_trade_stats_for_order(order_id: Uuid) {
    let conn = &get_pg_connection();
    let date = Utc::today().naive_utc();
    conn.build_transaction()
        .run::<_, diesel::result::Error, _>(|| {
            let mut order = Order::load_pk(&order_id).unwrap();
            if diesel::insert_into(ts::table)
                .values(
                    order
                        .manifest
                        .wtb
                        .drain(..)
                        .map(|oi| TradeStatistic {
                            item_code: oi.item_code,
                            buy: true,
                            quantity: oi.quantity as i64,
                            date,
                        })
                        .chain(order.manifest.wts.drain(..).map(|oi| TradeStatistic {
                            item_code: oi.item_code,
                            buy: false,
                            quantity: oi.quantity as i64,
                            date,
                        }))
                        .collect::<Vec<TradeStatistic>>(),
                )
                .on_conflict((ts::item_code, ts::buy, ts::date))
                .do_update()
                .set(ts::quantity.eq(ts::quantity + excluded(ts::quantity)))
                .execute(conn)
                .is_err()
            {
                Err(RollbackTransaction)
            } else {
                Ok(())
            }
        });
}
