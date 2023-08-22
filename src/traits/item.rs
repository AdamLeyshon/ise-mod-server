use std::borrow::Borrow;
use std::collections::HashSet;

use diesel::SaveChangesDsl;
use itsdangerous::Signer;

use crate::crypto::{hash_short_identity_string, sign_string};
use crate::db::models::bank::BankBalance;
use crate::db::models::inventory::Inventory;
use crate::db::models::order::Order;
use crate::db::Ppc;
use crate::packets::order::{OrderItem, OrderStatusEnum};
use crate::packets::tradable::{ColonyTradable, Tradable};
use crate::structs::bank_balance::get_bank_balance;
use crate::structs::inventory::{get_inventory, update_stock};
use crate::traits::numerical::CanRound;
use bigdecimal::{Signed, ToPrimitive};

pub trait ItemCodeComputable {
    fn generate_item_code(&self) -> String;
    fn get_version_code(&self) -> String;
    fn populate_identity_values(&mut self);
}

impl ItemCodeComputable for ColonyTradable {
    fn generate_item_code(&self) -> String {
        make_item_code(
            self.thing_def.clone(),
            if self.quality > 0 {
                Some(self.quality)
            } else {
                None
            },
            if !self.stuff.is_empty() {
                Some(self.stuff.clone())
            } else {
                None
            },
        )
    }
    fn get_version_code(&self) -> String {
        make_version_string(&self.generate_item_code(), self.base_value)
    }

    fn populate_identity_values(&mut self) {
        panic!("Not implemented")
    }
}

impl ItemCodeComputable for Inventory {
    fn generate_item_code(&self) -> String {
        make_item_code(
            self.thing_def.clone(),
            self.quality.clone(),
            self.stuff.clone(),
        )
    }
    fn get_version_code(&self) -> String {
        make_version_string(&self.item_code, self.base_value.to_f32().unwrap())
    }

    fn populate_identity_values(&mut self) {
        self.item_code = self.generate_item_code();
        self.version = self.get_version_code();
    }
}

pub fn make_version_string(item_code: &String, base_value: f32) -> String {
    let mut version = item_code.clone();
    version.push_str(&*format!("{:.2}", base_value));
    hash_short_identity_string(version)
}

pub trait HasItemCode {
    fn get_item_code(&self) -> &String;
}

pub trait HasThingDef {
    fn get_thing_def(&self) -> &String;
}

pub trait HasQuantity {
    fn get_quantity(&self) -> i64;
}

pub trait MakeTradable {
    fn make_tradable<S>(self, signer: &S) -> Tradable
    where
        S: Signer;
}

impl MakeTradable for Inventory {
    /// Convert from Inventory row to Protobuf message data
    fn make_tradable<S>(self, signer: &S) -> Tradable
    where
        S: Signer,
    {
        Tradable {
            thing_def: self.thing_def,
            item_code: sign_string(self.item_code, signer),
            quality: self.quality.unwrap_or(0),
            quantity: self.quantity,
            minified: self.minified,
            base_value: self.base_value.round_2dp().to_f32().unwrap(),
            we_buy_at: if self.buy_at.is_positive() {
                self.buy_at.round_2dp().to_f32().unwrap()
            } else {
                self.base_value.round_2dp().to_f32().unwrap()
            },
            we_sell_at: if self.sell_at.is_positive() {
                self.sell_at.round_2dp().to_f32().unwrap()
            } else {
                self.base_value.round_2dp().to_f32().unwrap()
            },
            stuff: self.stuff.unwrap_or(String::new()),
            weight: self.weight.round_2dp().to_f32().unwrap(),
        }
    }
}

pub(crate) trait Rollback {
    fn rollback(&mut self, conn: &Ppc) -> Result<(), ()>;
}

impl Rollback for Order {
    fn rollback(&mut self, conn: &Ppc) -> Result<(), ()> {
        let mut inventory_wanted =
            HashSet::<&String>::with_capacity(self.manifest.wts.len() + self.manifest.wtb.len());

        for item in self.manifest.wtb.iter().chain(self.manifest.wts.iter()) {
            inventory_wanted.insert(&item.item_code);
        }

        let mut inventory = get_inventory(inventory_wanted, &conn);
        let mut bank_balance =
            get_bank_balance(self.colony_id, self.manifest.currency.into(), &conn)?;

        // We flip the order of WTS and WTB in order to undo the stock changes
        update_stock(
            &self.manifest.wtb,
            &self.manifest.wts,
            &mut inventory,
            &conn,
        )?;

        // Add the inverse of what we added last time, clamp to zero.
        bank_balance.balance =
            (bank_balance.balance + (self.manifest.balance_adjustment * -1)).max(0);

        // Save changes to bank balance
        bank_balance
            .save_changes::<BankBalance>(&**conn)
            .map_err(|_| ())?;

        // Update the order itself
        self.status = OrderStatusEnum::Reversed.into();
        self.save_changes::<Order>(&**conn).map_err(|_| ())?;

        Ok(())
    }
}

pub trait ValidateItemSignature {
    fn validate_item_code<S>(&mut self, signer: &S) -> Result<(), ()>
    where
        S: Signer;
}

impl ValidateItemSignature for OrderItem {
    fn validate_item_code<S>(&mut self, signer: &S) -> Result<(), ()>
    where
        S: Signer,
    {
        self.item_code = match signer.unsign(self.item_code.borrow()) {
            Ok(value) => String::from(value),
            Err(_) => {
                return Err(());
            }
        };
        Ok(())
    }
}

pub fn make_item_code(thing_def: String, quality: Option<i32>, stuff: Option<String>) -> String {
    let mut data: String = String::new();
    data.push_str(&*thing_def);
    if let Some(quality) = quality {
        data.push_str(&*(quality.to_string()));
    }
    if let Some(stuff) = stuff {
        data.push_str(&*stuff);
    }
    hash_short_identity_string(data)
}
