use crate::request_helpers::*;
use actix_web::*;
use actix_web::{web, HttpResponse};
use chrono::{Duration, Utc};
use diesel::{sql_query, ExpressionMethods, RunQueryDsl};
use itertools::Itertools;
use uuid::Uuid;

use crate::crypto::{generate_random_alphanum_string, generate_v4_uuid, parse_uuid};
use crate::db::models::bind::ClientBind;
use crate::db::models::colony_tradable::ColonyTradables;
use crate::db::models::inventory::Inventory;
use crate::db::models::inventory_promise::InventoryPromise;
use crate::db::{get_pg_connection, Ppc};
use crate::packets::inventory::{
    ActivatePromiseReply, ActivatePromiseRequest, GeneratePromiseReply, GeneratePromiseRequest,
    InventoryReply, InventoryRequest,
};
use crate::packets::tradable::Tradable;
use crate::structs::api_config::LockedApiConfig;
use crate::structs::bank_balance::get_bank_balance;
use crate::structs::binds::ClientIdGuard;
use crate::structs::colony::validate_ownership_and_fetch;
use crate::structs::general::DbPkLoadable;
use crate::structs::inventory_promise::{
    get_promise_for_colony,
};
use crate::traits::item::MakeTradable;
use actix_web::web::Data;

pub fn config() -> Scope {
    web::scope("/inventory")
        .guard(guard::Header("content-type", "application/protobuf"))
        .guard(ClientIdGuard())
        .route("/", web::post().to(action_post))
        .route("/activate", web::post().to(action_activate))
        .route("/promise", web::post().to(action_make_promise))
}

pub async fn action_make_promise(
    _: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<GeneratePromiseRequest>,
) -> Result<HttpResponse> {
    validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind)
        .ok_or(())
        .and_then(|c| {
            let conn = &get_pg_connection();
            Ok(generate_promise(c.colony_id, conn))
        })
        .map(|p| {
            HttpResponse::Ok().protobuf(GeneratePromiseReply {
                inventory_promise_id: p.promise_id.to_string(),
                inventory_promise_expires: p.expiry_date.timestamp(),
            })
        })
        .unwrap_or(Ok(HttpResponse::BadRequest().finish()))
}

pub async fn action_activate(
    _: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<ActivatePromiseRequest>,
) -> Result<HttpResponse> {
    HttpResponse::Ok().protobuf(
        validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind)
            .ok_or(())
            .and_then(|c| parse_uuid(&packet.inventory_promise_id).map(|p| (c, p)))
            .and_then(|(c, p)| {
                let conn = &get_pg_connection();
                activate_promise(c.colony_id, p, conn).map_err(|_| ())
            })
            .map(|p| ActivatePromiseReply {
                success: true,
                inventory_promise_id: p.promise_id.to_string(),
                inventory_promise_expires: p.expiry_date.timestamp(),
            })
            .unwrap_or(ActivatePromiseReply {
                success: false,
                inventory_promise_id: "".to_string(),
                inventory_promise_expires: 0,
            }),
    )
}

pub async fn action_post(
    req: HttpRequest,
    bind: ClientBind,
    packet: ProtoBuf<InventoryRequest>,
) -> Result<HttpResponse> {
    let colony = match validate_ownership_and_fetch(None, Some(&packet.colony_id), &bind) {
        None => {
            return Ok(HttpResponse::BadRequest().finish());
        }
        Some(value) => value,
    };

    // Load the relevant colony data
    let colony_tradables = ColonyTradables::load_pk(&colony.colony_id);

    // Do we have a tradables list for this Colony?
    if colony_tradables.is_err() {
        return Ok(HttpResponse::UnprocessableEntity().finish());
    }

    // Unwrap data and get current time
    let colony_tradables = colony_tradables.unwrap();

    let now = Utc::now().naive_utc();

    // Make sure they updated the tradables recently.
    if (colony_tradables.update_date - now).num_minutes() > 5 {
        return Ok(HttpResponse::UnprocessableEntity().finish());
    }

    let conn = &get_pg_connection();

    // Create new promise
    // TODO: Configuration options for expiry times
    let promise = if !&packet.continue_existing_promise {
        generate_promise(colony.colony_id, conn)
    } else {
        match get_promise_for_colony(colony.colony_id) {
            Ok(p) => p,
            Err(_) => {
                return Ok(HttpResponse::UnprocessableEntity().finish());
            }
        }
    };

    // Load tradable data from Inventory table
    let tradables = get_inventory_for_colony(colony.colony_id, conn, promise.private_key.clone());

    let bank_balance = if let Ok(balance) = get_bank_balance(colony.colony_id, 0, conn) {
        balance
    } else {
        return Ok(HttpResponse::InternalServerError().finish());
    };

    let lock = req.app_data::<Data<LockedApiConfig>>().unwrap().read();
    let config = lock.as_ref().unwrap();

    // TODO: Configuration options for charges
    HttpResponse::Ok().protobuf(InventoryReply {
        items: tradables,
        inventory_promise_id: promise.promise_id.to_string(),
        inventory_promise_expires: promise.expiry_date.timestamp(),
        collection_charge_per_kg: config.config_data.delivery.collect_cost_per_kg as i32,
        delivery_charge_per_kg: config.config_data.delivery.delivery_cost_per_kg as i32,
        account_balance: bank_balance.balance,
    })
}

fn generate_promise(colony_id: Uuid, conn: &Ppc) -> InventoryPromise {
    use crate::db::schema::inventory_promises as promise_schema;

    let promise = InventoryPromise {
        colony_id,
        promise_id: generate_v4_uuid(),
        private_key: generate_random_alphanum_string(32),
        expiry_date: Utc::now().naive_utc() + Duration::minutes(5),
        activated: false,
    };

    diesel::insert_into(promise_schema::table)
        .values(&promise)
        .on_conflict(promise_schema::colony_id)
        .do_update()
        .set(&promise)
        .execute(conn)
        .expect("Failed to generate inventory promise");

    promise
}

fn activate_promise(
    colony_id: Uuid,
    promise_id: Uuid,
    conn: &Ppc,
) -> Result<InventoryPromise, String> {
    use crate::db::schema::inventory_promises as promise_schema;

    diesel::update(promise_schema::table)
        .set((
            promise_schema::expiry_date.eq(Utc::now().naive_utc() + Duration::minutes(5)),
            promise_schema::activated.eq(true),
        ))
        .filter(promise_schema::colony_id.eq(colony_id))
        .filter(promise_schema::promise_id.eq(promise_id))
        .filter(promise_schema::activated.eq(false))
        .get_result::<InventoryPromise>(conn)
        .map_err(|_| "Failed to find provided promise ID".to_owned())
}

fn get_inventory_for_colony(colony_id: Uuid, conn: &Ppc, secret_key: String) -> Vec<Tradable> {
    use itsdangerous::default_builder;
    let inventory_query = "SELECT j.*
    FROM colony_tradables as cto \
    LEFT JOIN LATERAL ( \
    select ct.code as item_code, \
    thing_def, \
    quality, \
    quantity, \
    minified, \
    base_value, \
    buy_at, \
    sell_at, \
    stuff, \
    weight, \
    version \
    from jsonb_array_elements_text(cto.tradables) as ct(code) \
    INNER JOIN inventory i on i.item_code = ct.code \
    ) j on true \
    WHERE colony_id = $1";

    // We send them everything even if none in stock, because otherwise
    // it won't show in the UI since we only iterate rows from the server
    // Saves having to combine two lists on the client side.

    let inventory: Vec<Inventory> = match sql_query(inventory_query)
        .bind::<diesel::sql_types::Uuid, _>(colony_id)
        .get_results(conn)
    {
        Err(e) => {
            warn!("Suspicious error when getting inventory, {}", e);
            Vec::new()
        }
        Ok(data) => data,
    };

    // Create a signer using the default builder, and an arbitrary secret key.
    let signer = default_builder(secret_key).build();
    inventory
        .into_iter()
        .map(|inv| inv.make_tradable(&signer))
        .collect_vec()
}
