use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use diesel::dsl::any;
use diesel::prelude::*;

use crate::db::models::inventory::Inventory;
use crate::db::Ppc;
use crate::packets::order::OrderItem;
use crate::packets::tradable::ColonyTradable;
use crate::structs::api_config::API_CONFIG_ARC;
use crate::structs::order::OrderStats;
use crate::traits::item::ItemCodeComputable;

impl From<ColonyTradable> for Inventory {
    fn from(ct: ColonyTradable) -> Self {
        Inventory {
            item_code: ct.generate_item_code(),
            thing_def: ct.thing_def.clone(),
            quality: if ct.quality > 0 {
                Some(ct.quality)
            } else {
                None
            },
            quantity: 0,
            minified: ct.minified,
            base_value: BigDecimal::from(ct.base_value),
            buy_at: BigDecimal::from(ct.base_value),
            sell_at: BigDecimal::from(ct.base_value),
            stuff: if ct.stuff.is_empty() {
                None
            } else {
                Some(ct.stuff.clone())
            },
            weight: BigDecimal::from(ct.weight),
            version: ct.get_version_code(),
        }
    }
}

pub fn create_silver_inventory_item() -> Inventory {
    let mut silver = Inventory {
        item_code: "".to_string(),
        thing_def: "Silver".to_string(),
        quality: None,
        quantity: 0,
        minified: false,
        base_value: BigDecimal::from(1),
        buy_at: BigDecimal::from(1),
        sell_at: BigDecimal::from(1),
        stuff: None,
        weight: BigDecimal::from(0.008),
        version: "".to_string(),
    };
    silver.populate_identity_values();
    silver
}

use crate::traits::numerical::CanRound;
use bigdecimal::BigDecimal;
use lazy_static::lazy_static;
lazy_static! {
    pub static ref SILVER_ITEM: Inventory = create_silver_inventory_item();
}

pub fn get_inventory(mut item_codes: HashSet<&String>, conn: &Ppc) -> HashMap<String, Inventory> {
    use crate::db::schema::inventory as schema;
    let mut results: Vec<Inventory> = schema::table
        .filter(schema::item_code.eq(any(item_codes.drain().collect::<Vec<&String>>())))
        .get_results(conn)
        .expect("Failed colony<->inventory load query");
    let mut hash = HashMap::<String, Inventory>::with_capacity(results.len());
    for item in results.drain(..) {
        hash.insert(item.item_code.clone(), item);
    }
    hash
}

pub fn update_stock(
    wts: &Vec<OrderItem>,
    wtb: &Vec<OrderItem>,
    db_inventory: &mut HashMap<String, Inventory>,
    conn: &Ppc,
) -> Result<(OrderStats, Vec<OrderItem>), ()> {
    let mut os = OrderStats::default();
    let mut out_of_stock = Vec::<OrderItem>::new();
    for item in wts {
        let val = db_inventory.get_mut(&item.item_code).unwrap();
        let quantity = BigDecimal::from(item.quantity);
        val.quantity += item.quantity;
        os.total_sell_weight += &val.weight * &quantity;
        os.total_sell_cost += ((&val.buy_at / 100f32) * BigDecimal::from(item.health)) * &quantity;
    }
    for item in wtb {
        let stock = db_inventory.get_mut(&item.item_code).unwrap();
        // Are we completely out of stock?
        if stock.quantity < item.quantity {
            // Refuse the sale, we'll refund them later
            out_of_stock.push(item.clone());
        } else {
            let quantity = BigDecimal::from(item.quantity);
            // Clamp minimum value at Zero items in stock
            stock.quantity = (stock.quantity - item.quantity).max(0);
            os.total_buy_weight += &stock.weight * &quantity;
            os.total_buy_cost +=
                ((&stock.sell_at / 100f32) * BigDecimal::from(item.health)) * &quantity;
        }
    }
    for value in db_inventory.values() {
        if value.save_changes::<Inventory>(conn.deref()).is_err() {
            return Err(());
        }
    }

    let read_lock = API_CONFIG_ARC.read();
    let config = read_lock.as_ref().unwrap();

    // Add delivery/collection fees whilst rounding up/down to nearest integer
    os.total_sell_weight = os.total_sell_weight.round_2dp();
    os.total_buy_weight = os.total_buy_weight.round_2dp();
    os.total_buy_cost +=
        &os.total_buy_weight * BigDecimal::from(config.config_data.delivery.delivery_cost_per_kg);
    os.total_buy_cost +=
        &os.total_sell_weight * BigDecimal::from(config.config_data.delivery.collect_cost_per_kg);
    os.total_buy_cost = os.total_buy_cost.round_2dp();
    os.total_sell_cost = os.total_sell_cost.round_2dp();
    Ok((os, out_of_stock))
}
