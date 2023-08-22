use crate::db::models::inventory_staging::ColonyInventoryStaging;
use crate::db::models::new_inventory::NewInventory;
use crate::traits::from::FromWithTime;
use crate::traits::item::{HasItemCode, HasThingDef};
use chrono::NaiveDateTime;

impl FromWithTime<ColonyInventoryStaging> for NewInventory {
    fn from_wt(ct: ColonyInventoryStaging, date: NaiveDateTime) -> Self {
        NewInventory {
            version: ct.version,
            item_code: ct.item_code,
            thing_def: ct.thing_def,
            quality: ct.quality,
            minified: ct.minified,
            base_value: ct.base_value,
            stuff: ct.stuff,
            weight: ct.weight,
            date_added: date,
        }
    }
}

impl HasThingDef for NewInventory {
    fn get_thing_def(&self) -> &String {
        &self.thing_def
    }
}

impl HasItemCode for NewInventory {
    fn get_item_code(&self) -> &String {
        &self.item_code
    }
}

impl<'a> HasThingDef for &'a NewInventory {
    fn get_thing_def(&self) -> &String {
        &self.thing_def
    }
}

impl<'a> HasItemCode for &'a NewInventory {
    fn get_item_code(&self) -> &String {
        &self.item_code
    }
}
