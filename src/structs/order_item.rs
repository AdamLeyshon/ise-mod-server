use crate::db::models::inventory::Inventory;
use crate::packets::order::OrderItem;
use crate::traits::item::{HasItemCode, HasQuantity};

impl HasItemCode for OrderItem {
    fn get_item_code(&self) -> &String {
        &self.item_code
    }
}

impl HasQuantity for OrderItem {
    fn get_quantity(&self) -> i64 {
        self.quantity as i64
    }
}

impl From<Inventory> for OrderItem {
    fn from(inv: Inventory) -> Self {
        OrderItem {
            item_code: inv.item_code,
            quantity: 0,
            health: 100f32,
        }
    }
}
