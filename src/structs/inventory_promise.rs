use crate::db::models::inventory_promise::InventoryPromise;
use crate::structs::general::DbPkLoadable;
use chrono::Utc;
use uuid::Uuid;

make_pk_loadable!(
    InventoryPromise,
    Uuid,
    crate::db::schema::inventory_promises
);

pub fn validate_promise_id(
    colony_id: Uuid,
    promise_id: Uuid,
) -> Result<InventoryPromise, InventoryPromiseError> {
    // Check that the colony has a promise, it's the same ID and that it hasn't expired
    get_promise_for_colony(colony_id).and_then(|ip| {
        if ip.promise_id == promise_id {
            Ok(ip)
        } else {
            Err(InventoryPromiseError::Mismatched)
        }
    })
}

#[derive(Debug, ToString)]
pub enum InventoryPromiseError {
    NotFound,
    Expired,
    Deactivated,
    Mismatched,
}

pub fn get_promise_for_colony(colony_id: Uuid) -> Result<InventoryPromise, InventoryPromiseError> {
    // Check that the colony has a promise, it's the same ID and that it hasn't expired
    InventoryPromise::load_pk(&colony_id)
        .map_err(|_| InventoryPromiseError::NotFound)
        .and_then(|ip| {
            if ip.expiry_date < Utc::now().naive_utc() {
                Err(InventoryPromiseError::Expired)
            } else if !ip.activated {
                Err(InventoryPromiseError::Deactivated)
            } else {
                Ok(ip)
            }
        })
}
