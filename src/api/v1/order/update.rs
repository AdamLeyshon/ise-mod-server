use std::convert::TryFrom;
use std::ops::Deref;

use crate::request_helpers::*;
use actix_web::*;
use diesel::prelude::*;
use diesel::SaveChangesDsl;
use diesel::{ExpressionMethods, QueryDsl};
//use http_api_problem::*;

use crate::crypto::parse_uuid;
use crate::db::get_pg_connection;
use crate::db::models::bind::ClientBind;
use crate::db::models::colony::Colony;
use crate::db::models::order::{Order, OrderNoManifest};
use crate::db::schema::orders as schema;
use crate::packets::order::{OrderStatusEnum, OrderStatusReply, OrderUpdateRequest};
use crate::stats::order::update_trade_stats_for_order;
use crate::structs::colony::validate_ownership_and_fetch;
use crate::structs::general::ONE_HOUR_TICKS;
use std::thread::spawn;

pub async fn action_update(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<OrderUpdateRequest>,
) -> Result<HttpResponse> {
    if let Some(mut colony) = validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind) {
        let conn = &get_pg_connection();
        let order_id = match parse_uuid(&*packet.order_id) {
            Ok(uuid) => uuid,
            Err(_) => return Ok(HttpResponse::BadRequest().finish()),
        };
        let mut order: OrderNoManifest = match schema::table
            .select((
                schema::order_id,
                schema::colony_id,
                schema::status,
                schema::start_tick,
                schema::end_tick,
                schema::order_stats,
                schema::create_date,
                schema::update_date,
            ))
            .filter(schema::colony_id.eq(&colony.colony_id))
            .filter(schema::order_id.eq(order_id))
            .get_result(conn)
        {
            Err(diesel::NotFound) => {
                return Ok(HttpResponse::BadRequest().finish());
            }
            Err(_) => {
                return Ok(HttpResponse::InternalServerError().finish());
            }
            Ok(v) => v,
        };

        if packet.colony_tick < colony.tick {
            return Ok(HttpResponse::BadRequest().finish());
            // return Ok(HttpApiProblem::new(StatusCode::BAD_REQUEST)
            //     .title("Problem updating order")
            //     .detail("Colony tick de-sync, do a Colony Update first.")
            //     .instance(format!("{}", order_id))
            //     .to_actix_response());
        } else {
            colony.tick = packet.colony_tick;
        }

        // Parse the int32 into an Enum, Easier to work with.
        let status_enum = match OrderStatusEnum::try_from(packet.status) {
            Ok(v) => v,
            Err(_) => {
                return Ok(HttpResponse::BadRequest().finish());
            }
        };

        // Enter state machine here
        order.status = match OrderStatusEnum::try_from(order.status).unwrap() {
            // If the status is Placed, the only option is OutForDelivery
            OrderStatusEnum::Placed => {
                match status_enum {
                    OrderStatusEnum::OutForDelivery => {
                        // Don't all allow status change until
                        // at least 6 in-game hours before delivery
                        if colony.tick > (order.end_tick - (ONE_HOUR_TICKS * 6)) {
                            status_enum.into()
                        } else {
                            return Ok(HttpResponse::BadRequest().finish());
                        }
                    }
                    _ => {
                        return Ok(HttpResponse::BadRequest().finish());
                    }
                }
            }
            // If it's OutForDelivery, only two statuses, Failed and Delivered
            OrderStatusEnum::OutForDelivery => match status_enum {
                OrderStatusEnum::Delivered => {
                    // Update trade stats in a separate thread
                    spawn(move || update_trade_stats_for_order(order_id.clone()));
                    status_enum.into()
                }
                OrderStatusEnum::Failed => {
                    // Rollback the order?
                    status_enum.into()
                }
                _ => {
                    return Ok(HttpResponse::BadRequest().finish());
                }
            },
            // Any other state is invalid for updates.
            _ => {
                return Ok(HttpResponse::BadRequest().finish());
            }
        };

        // Save the new tick value for the Colony.
        if colony.save_changes::<Colony>(conn.deref()).is_err() {
            return Ok(HttpResponse::InternalServerError().finish());
        };

        // Save the new status.
        if order.save_changes::<Order>(conn.deref()).is_err() {
            return Ok(HttpResponse::InternalServerError().finish());
        };

        HttpResponse::Ok().protobuf(OrderStatusReply::from(order))
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
