use crate::request_helpers::*;
use actix_web::*;
use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl};
use itertools::Itertools;

use crate::db::get_pg_connection;
use crate::db::models::bind::ClientBind;
use crate::db::models::order::Order;
use crate::db::schema::orders as schema;
use crate::packets::order::{OrderListReply, OrderListRequest, OrderStatusEnum, OrderStatusReply};
use crate::structs::colony::validate_ownership_and_fetch;

pub async fn action_get(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<OrderListRequest>,
) -> Result<HttpResponse> {
    if let Some(colony) = validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind) {
        let mut query = schema::table
            .filter(schema::colony_id.eq(&colony.colony_id))
            .into_boxed();

        // If they don't want everything, filter by outstanding orders
        if !packet.any {
            query = query.filter(
                schema::status
                    .eq(i32::from(OrderStatusEnum::Placed))
                    .or(schema::status.eq(i32::from(OrderStatusEnum::OutForDelivery))),
            );
        }

        let mut orders: Vec<Order> = match query.get_results(&get_pg_connection()) {
            Err(_) => {
                return Ok(HttpResponse::InternalServerError().finish());
            }
            Ok(v) => v,
        };

        HttpResponse::Ok().protobuf(OrderListReply {
            orders: orders
                .drain(..)
                .map_into::<OrderStatusReply>()
                .collect_vec(),
        })
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
