use crate::db::models::inventory_staging::ColonyInventoryStaging;
use crate::packets::tradable::ColonyTradable;
use crate::traits::from::FromWithColonyUuid;
use crate::traits::item::{make_version_string, HasItemCode, HasThingDef, ItemCodeComputable};
use bigdecimal::BigDecimal;
use uuid::Uuid;

impl FromWithColonyUuid<ColonyTradable> for ColonyInventoryStaging {
    fn from_with_uuid(ct: ColonyTradable, colony_id: Uuid) -> Self {
        let item_code = ct.generate_item_code();
        ColonyInventoryStaging {
            colony_id,
            version: make_version_string(&item_code, ct.base_value),
            item_code, // Will be populated by get_version_code
            thing_def: ct.thing_def,
            quality: if ct.quality > 0 {
                Some(ct.quality)
            } else {
                None
            },
            minified: ct.minified,
            base_value: BigDecimal::from(ct.base_value),
            stuff: if ct.stuff.is_empty() {
                None
            } else {
                Some(ct.stuff)
            },
            weight: BigDecimal::from(ct.weight),
        }
    }
}

impl HasThingDef for ColonyInventoryStaging {
    fn get_thing_def(&self) -> &String {
        &self.thing_def
    }
}

impl HasItemCode for ColonyInventoryStaging {
    fn get_item_code(&self) -> &String {
        &self.item_code
    }
}

impl<'a> HasThingDef for &'a ColonyInventoryStaging {
    fn get_thing_def(&self) -> &String {
        &self.thing_def
    }
}

impl<'a> HasItemCode for &'a ColonyInventoryStaging {
    fn get_item_code(&self) -> &String {
        &self.item_code
    }
}
