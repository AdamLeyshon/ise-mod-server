use crate::db::models::inventory::Inventory;
use crate::packets::tradable::ColonyTradable;
use crate::structs::general::DbPkLoadable;
use crate::traits::item::{HasItemCode, HasThingDef};

make_pk_loadable!(Inventory, String, crate::db::schema::inventory);

impl HasItemCode for Inventory {
    fn get_item_code(&self) -> &String {
        &self.item_code
    }
}

impl HasThingDef for Inventory {
    fn get_thing_def(&self) -> &String {
        &self.thing_def
    }
}

impl HasThingDef for ColonyTradable {
    fn get_thing_def(&self) -> &String {
        &self.thing_def
    }
}
