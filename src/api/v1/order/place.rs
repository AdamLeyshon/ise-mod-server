use std::collections::HashSet;
use std::convert::TryFrom;
use std::thread::spawn;

use crate::request_helpers::ProtoBuf;
use crate::request_helpers::*;
use actix_web::*;
use itsdangerous::default_builder;

use crate::db::models::bind::ClientBind;

use crate::crypto::parse_uuid;
use crate::db::get_pg_connection;
use crate::packets::common::CurrencyEnum;
use crate::packets::order::{
    OrderReply, OrderRequest, OrderRequestStatus, OrderStatusEnum, OrderStatusReply,
};
use crate::stats::order::update_trade_stats_for_order;
use crate::structs::bank_balance::get_bank_balance;
use crate::structs::colony::validate_ownership_and_fetch;
use crate::structs::inventory::get_inventory;
use crate::structs::order::OrderManifest;
use crate::structs::{bank_balance, inventory, inventory_promise, order};
use crate::traits::item::ValidateItemSignature;
use crate::traits::numerical::CanRound;
use bigdecimal::ToPrimitive;

pub async fn action_post(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<OrderRequest>,
) -> Result<HttpResponse> {
    let colony = match validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind) {
        None => {
            return Ok(HttpResponse::Forbidden().finish());
        }
        Some(c) => c,
    };

    if packet.colony_tick != colony.tick {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let promise_id = match parse_uuid(&*packet.inventory_promise_id) {
        Ok(uuid) => uuid,
        Err(_) => return Ok(HttpResponse::Gone().finish()),
    };

    let promise = match inventory_promise::validate_promise_id(colony.colony_id, promise_id) {
        Err(e) => {
            error!("Error processing order, Invalid Promise. C: {}, P: {}, S: {}",
                colony.colony_id,
                packet.inventory_promise_id,
                e.to_string()
            );
            return Ok(HttpResponse::Gone().finish());
        }
        Ok(ip) => ip,
    };

    // This can be done in parallel if needed later on with a threadpool
    // Verify the item codes haven't been changed and that they were signed with our Promise,
    // Drop out as soon as an error is detected.
    let signer = default_builder(promise.private_key).build();
    let mut wts = packet.0.want_to_sell;
    let mut wtb = packet.0.want_to_buy;
    let mut inventory_wanted = HashSet::<&String>::with_capacity(wts.len() + wtb.len());
    let currency = CurrencyEnum::try_from(packet.0.currency).unwrap();
    let additional_funds = packet.0.additional_funds;

    // We can't be sent things that aren't in our inventory,
    // Trying to do so causes the validation routine to fail here
    // Since all items are signed with a promise
    for item in wts.iter_mut().chain(wtb.iter_mut()) {
        if item.validate_item_code(&signer).is_err() {
            return Ok(HttpResponse::Forbidden().finish());
        }
        inventory_wanted.insert(&item.item_code);
    }

    // Now fetch all the inventory rows related to the items WTS/WTB
    let conn = &get_pg_connection();
    let mut db_inventory = get_inventory(inventory_wanted, conn);

    match conn
        .build_transaction()
        .read_committed()
        .run::<_, diesel::result::Error, _>(|| {
            let (os, out_of_stock) =
                match inventory::update_stock(&wts, &wtb, &mut db_inventory, conn) {
                    Err(_) => {
                        return Err(diesel::result::Error::RollbackTransaction);
                    }
                    Ok(v) => v,
                };

            let mut bank_balance = match get_bank_balance(colony.colony_id, currency.into(), conn) {
                Err(_) => {
                    return Err(diesel::result::Error::RollbackTransaction);
                }
                Ok(v) => v,
            };

            // Check if their bank balance will be positive after the transaction,
            // If not reject.
            if bank_balance.balance as f32
                + additional_funds as f32
                + (&os.total_sell_cost - &os.total_buy_cost)
                .round_2dp()
                .to_f32()
                .unwrap()
                < 0f32
            {
                // They'd still be in debt after selling every thing, reject it.
                return Err(diesel::result::Error::RollbackTransaction);
            }

            let (refunded, balance_adjustment) = match bank_balance::update_bank(
                &os,
                &db_inventory,
                additional_funds,
                if out_of_stock.is_empty() {
                    None
                } else {
                    Some(&out_of_stock)
                },
                &mut bank_balance,
                conn,
            ) {
                Err(_) => {
                    return Err(diesel::result::Error::RollbackTransaction);
                }
                Ok(v) => v,
            };

            let manifest = OrderManifest {
                wts,
                wtb,
                balance_adjustment,
                currency,
            };

            let order = match order::create_order(&os, &colony, manifest, colony.tick, None, conn) {
                Err(_) => {
                    return Err(diesel::result::Error::RollbackTransaction);
                }
                Ok(v) => v,
            };

            let reply = Some(OrderStatusReply {
                order_id: order.order_id.to_string(),
                status: order.status,
                delivery_tick: order.end_tick,
                placed_tick: order.start_tick,
            });

            Ok((
                order,
                HttpResponse::Ok().protobuf(OrderReply {
                    data: reply,
                    status: if out_of_stock.len() > 0 {
                        OrderRequestStatus::AcceptedPartial.into()
                    } else {
                        OrderRequestStatus::AcceptedAll.into()
                    },
                    unavailable: out_of_stock,
                    refunded,
                    balance: bank_balance.balance,
                }),
            ))
        }) {
        Ok((order, response)) => {
            // Need to update trade stats for sell only orders
            // But we need to do it after the transaction is committed.
            if order.status == i32::from(OrderStatusEnum::Delivered) {
                let order_id = order.order_id;
                spawn(move || update_trade_stats_for_order(order_id));
            }
            response
        }
        Err(_) => HttpResponse::Ok().protobuf(OrderReply {
            data: None,
            status: OrderRequestStatus::Rejected.into(),
            unavailable: vec![],
            refunded: 0,
            balance: 0,
        }),
    }
}
