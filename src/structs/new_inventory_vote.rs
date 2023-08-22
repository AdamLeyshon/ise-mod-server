use macros::FieldCount;

use crate::db::models::inventory::{InventoryNoQuantity, InventoryOnlyStats, TempInventoryVote};
use bigdecimal::BigDecimal;

#[derive(Queryable, Debug, FieldCount)]
pub struct InventoryVote {
    pub item_code: String,
    pub thing_def: String,
    pub quality: Option<i32>,
    pub minified: bool,
    pub base_value: BigDecimal,
    pub stuff: Option<String>,
    pub weight: BigDecimal,
    pub version: String,
    pub votes: i64,
}

impl From<InventoryVote> for InventoryNoQuantity {
    fn from(iv: InventoryVote) -> Self {
        InventoryNoQuantity {
            item_code: iv.item_code,
            thing_def: iv.thing_def,
            quality: iv.quality,
            minified: iv.minified,
            base_value: iv.base_value.clone(),
            buy_at: iv.base_value.clone(),
            sell_at: iv.base_value,
            stuff: iv.stuff,
            weight: iv.weight,
            version: iv.version,
        }
    }
}

impl From<TempInventoryVote> for InventoryNoQuantity {
    fn from(iv: TempInventoryVote) -> Self {
        InventoryNoQuantity {
            item_code: iv.item_code,
            thing_def: iv.thing_def,
            quality: iv.quality,
            minified: iv.minified,
            base_value: iv.base_value.clone(),
            buy_at: iv.base_value.clone(),
            sell_at: iv.base_value,
            stuff: iv.stuff,
            weight: iv.weight,
            version: iv.version,
        }
    }
}

impl From<InventoryNoQuantity> for InventoryOnlyStats {
    fn from(iv: InventoryNoQuantity) -> Self {
        InventoryOnlyStats {
            item_code: iv.item_code,
            thing_def: iv.thing_def,
            minified: iv.minified,
            base_value: iv.base_value,
            weight: iv.weight,
            version: iv.version,
        }
    }
}
