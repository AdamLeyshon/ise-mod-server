use crate::request_helpers::*;
use actix_web::*;
use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl};

use crate::crypto::parse_uuid;
use crate::db::get_pg_connection;
use crate::db::models::bind::ClientBind;
use crate::db::models::order::Order;
use crate::db::schema::orders as schema;
use crate::packets::order::{OrderStatusReply, OrderStatusRequest};
use crate::structs::colony::validate_ownership_and_fetch;

pub async fn action_get(
    _req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<OrderStatusRequest>,
) -> Result<HttpResponse> {
    if let Some(colony) = validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind) {
        let order_uuid = match parse_uuid(&*packet.order_id) {
            Ok(uuid) => uuid,
            Err(_) => return Ok(HttpResponse::BadRequest().finish()),
        };
        let order: Order = match schema::table
            .filter(schema::colony_id.eq(&colony.colony_id))
            .filter(schema::order_id.eq(order_uuid))
            .get_result(&get_pg_connection())
        {
            Err(diesel::NotFound) => return Ok(HttpResponse::BadRequest().finish()),
            Err(_) => return Ok(HttpResponse::InternalServerError().finish()),
            Ok(v) => v,
        };

        HttpResponse::Ok().protobuf(OrderStatusReply::from(order))
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
