use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::db::models::inventory::Inventory;
use crate::db::models::new_inventory::NewInventory;
use crate::db::models::new_inventory_vote::NewInventoryVote;
use crate::db::models::price_tracker::PriceTracker;

pub trait InventoryToPT {
    fn i2pt(c: Inventory, now: &NaiveDateTime) -> Self;
}

impl InventoryToPT for PriceTracker {
    /// Convert from Inventory row to Protobuf message data
    fn i2pt(c: Inventory, now: &NaiveDateTime) -> Self {
        PriceTracker {
            item_code: c.item_code,
            value: c.base_value,
            create_date: *now,
        }
    }
}

pub trait FromWithTime<T> {
    fn from_wt(_: T, _: NaiveDateTime) -> Self;
}

pub trait FromWithColonyUuid<T> {
    fn from_with_uuid(_: T, _: Uuid) -> Self;
}

pub trait NewInventoryToVote {
    fn vote(client_bind_id: Uuid, colony_id: Uuid, item: NewInventory) -> Self;
}

impl NewInventoryToVote for NewInventoryVote {
    fn vote(client_bind_id: Uuid, colony_id: Uuid, item: NewInventory) -> Self {
        NewInventoryVote {
            client_bind_id,
            colony_id,
            version: item.version,
        }
    }
}
