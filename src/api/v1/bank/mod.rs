use std::collections::HashMap;

use crate::request_helpers::*;
use actix_web::*;
use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use strum::IntoEnumIterator;

use crate::db::get_pg_connection;
use crate::db::models::bank::BankBalance;
use crate::db::models::bind::ClientBind;
use crate::packets::bank::{BankDataReply, BankGetRequest, BankWithdrawReply, BankWithdrawRequest};
use crate::packets::common::CurrencyEnum;
use crate::packets::order::{OrderItem, OrderRequestStatus, OrderStatusReply};
use crate::structs::bank_balance::get_bank_balance;
use crate::structs::binds::ClientIdGuard;
use crate::structs::colony::validate_ownership_and_fetch;
use crate::structs::inventory::SILVER_ITEM;
use crate::structs::order;
use crate::structs::order::{OrderManifest, OrderStats};
use crate::traits::numerical::CanRound;
use bigdecimal::BigDecimal;
use std::convert::TryFrom;

pub fn config() -> Scope {
    let mut large_payload_size = ProtoBufConfig::default();
    large_payload_size.limit(10_000_000);

    web::scope("/bank")
        .guard(guard::Header("content-type", "application/protobuf"))
        .guard(ClientIdGuard())
        .route("/", web::post().to(action_get))
        .route("/withdraw", web::post().to(action_withdraw))
}

pub async fn action_get(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<BankGetRequest>,
) -> Result<HttpResponse> {
    let colony = match validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind) {
        None => {
            return Ok(HttpResponse::BadRequest().finish());
        }
        Some(value) => value,
    };
    let conn = &get_pg_connection();
    let mut bank_data = HashMap::<i32, i32>::new();
    for currency in CurrencyEnum::iter() {
        let currency_i32: i32 = currency.into();
        match get_bank_balance(colony.colony_id, currency_i32, conn) {
            Ok(data) => {
                bank_data.insert(currency_i32, data.balance);
            }
            Err(_) => {
                return Ok(HttpResponse::InternalServerError().finish());
            }
        };
    }
    HttpResponse::Ok().protobuf(BankDataReply { balance: bank_data })
}

pub async fn action_withdraw(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<BankWithdrawRequest>,
) -> Result<HttpResponse> {
    let colony = match validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind) {
        None => {
            return Ok(HttpResponse::BadRequest().finish());
        }
        Some(value) => value,
    };

    let conn = &get_pg_connection();
    match conn
        .build_transaction()
        .read_committed()
        .run::<Result<HttpResponse>, diesel::result::Error, _>(|| {
            let mut bank_balance = match get_bank_balance(colony.colony_id, packet.currency, conn) {
                Ok(data) => data,
                Err(_) => {
                    return Err(diesel::result::Error::RollbackTransaction);
                }
            };

            if packet.amount > 0 && packet.amount <= bank_balance.balance {
                bank_balance.balance -= packet.amount;
                if bank_balance.save_changes::<BankBalance>(&**conn).is_err() {
                    return Err(diesel::result::Error::RollbackTransaction);
                }
            } else {
                return Err(diesel::result::Error::RollbackTransaction);
            };

            let mut order_stats = OrderStats::default();
            let silver = &SILVER_ITEM;
            let mut oi_silver = OrderItem {
                item_code: silver.item_code.clone(),
                quantity: 0,
                health: 100f32,
            };

            let silver_amount: BigDecimal = packet.amount.into();
            order_stats.total_buy_cost += &silver_amount;
            order_stats.total_buy_cost = order_stats.total_buy_cost.round_2dp();
            order_stats.total_buy_weight += (&silver.weight * &silver_amount).round_2dp();
            oi_silver.quantity = packet.amount;

            let manifest = OrderManifest {
                wts: vec![],
                wtb: vec![oi_silver],
                balance_adjustment: packet.amount * -1,
                currency: CurrencyEnum::try_from(bank_balance.currency).unwrap(),
            };

            let order = match order::create_order(
                &order_stats,
                &colony,
                manifest,
                colony.tick,
                Some(colony.tick),
                conn,
            ) {
                Err(_) => {
                    return Err(diesel::result::Error::RollbackTransaction);
                }
                Ok(v) => v,
            };

            Ok(HttpResponse::Ok().protobuf(BankWithdrawReply {
                data: Some(OrderStatusReply {
                    order_id: order.order_id.to_string(),
                    status: order.status.into(),
                    delivery_tick: order.end_tick,
                    placed_tick: order.start_tick,
                }),
                status: OrderRequestStatus::AcceptedAll.into(),
                balance: bank_balance.balance,
            }))
        }) {
        Ok(response) => response,
        Err(_) => HttpResponse::Ok().protobuf(BankWithdrawReply {
            data: None,
            status: OrderRequestStatus::Rejected.into(),
            balance: 0,
        }),
    }
}
